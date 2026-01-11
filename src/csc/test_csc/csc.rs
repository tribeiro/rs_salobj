//! Implement the Test CSC.
//!
//! The TestCSC is a component that, as the name suggests, is used for
//! testing, both of the basic library and of deployments.
//!
//! This component implements the basic behavior expected of a CSC; it
//! contains a set of commands that can be executed while the CSC is in
//! Enabled and should be rejected if the CSC is in any other state. One can
//! transition this component through the different states sending the regular
//! commands; start, enable, disable, standby and exit control.
//!
//! Once in Enabled the CSC will accept the enabled commands and execute some
//! operations with the provided data.

use std::collections::{HashMap, HashSet};

use apache_avro::{from_value, types::Value};
use handle_command::handle_command;

use tokio::{
    sync::{mpsc, watch},
    task,
    time::{sleep, timeout, Duration},
};

use crate::{
    controller::Controller,
    csc::{
        base_csc::{BaseCSC, HEARTBEAT_TIME},
        test_csc::topics::{arrays::Arrays, scalars::Scalars, telemetry::TestTelemetry},
    },
    domain::Domain,
    error::errors::{SalObjError, SalObjResult},
    generics::{
        disable::Disable, empty_topic::EmptyTopic, enable::Enable, exit_control::ExitControl,
        heartbeat::Heartbeat, standby::Standby, start::Start, summary_state::SummaryState,
    },
    sal_enums::State,
    sal_info::SalInfo,
    topics::{
        base_sal_topic::BaseSALTopic, controller_command::ControllerCommand,
        controller_command_ack::ControllerCommandAck, write_topic::WriteTopic,
    },
    utils::{command_ack::CommandAck, types::WriteTopicSet},
};

use super::topics::wait::Wait;

struct CmdData {
    pub name: String,
    pub data: Value,
}

type CmdPayload = (CmdData, mpsc::Sender<CommandAck>);
type CommandAckResult = (CommandAck, mpsc::Sender<CommandAck>);

#[derive(Default)]
struct TelemetryPayload {
    pub name: String,
    pub data: TestTelemetry,
}

pub struct TestCSC<'a> {
    summary_state: State,
    domain: Domain,
    index: isize,
    controller: Controller<'a>,
    controller_command_ack: Option<ControllerCommandAck>,
    heartbeat_task: Option<task::JoinHandle<()>>,
    telemetry_loop_task: Option<task::JoinHandle<()>>,
    command_sender: mpsc::Sender<CmdPayload>,
    command_receiver: mpsc::Receiver<CmdPayload>,
    telemetry_sender: watch::Sender<TelemetryPayload>,
    telemetry_receiver: watch::Receiver<TelemetryPayload>,
}

impl<'a> TestCSC<'a> {
    pub async fn new(index: isize) -> SalObjResult<TestCSC<'a>> {
        let mut domain = Domain::new();
        let controller = Controller::new(&mut domain, "Test", index).await?;
        let (command_sender, command_receiver): (
            mpsc::Sender<CmdPayload>,
            mpsc::Receiver<CmdPayload>,
        ) = mpsc::channel(32);

        let (telemetry_sender, telemetry_receiver): (
            watch::Sender<TelemetryPayload>,
            watch::Receiver<TelemetryPayload>,
        ) = watch::channel(TelemetryPayload::default());

        Ok(TestCSC {
            summary_state: State::Standby,
            domain,
            index,
            controller,
            controller_command_ack: None,
            heartbeat_task: None,
            telemetry_loop_task: None,
            command_sender,
            command_receiver,
            telemetry_sender,
            telemetry_receiver,
        })
    }

    /// Start the CSC.
    ///
    /// This method should run only once after instantiating the CSC and will
    /// setup a series of background tasks that operates the CSC.
    pub async fn start(&mut self) {
        if let Err(err) = self.update_summary_state().await {
            log::error!("Failed to write summary state: {err:?}");
            return;
        };

        let sal_info = SalInfo::new("Test", self.index).unwrap();

        let mut heartbeat_writer = WriteTopic::new("logevent_heartbeat", &sal_info, &self.domain);

        let heartbeat_task = task::spawn(async move {
            let origin = heartbeat_writer.get_origin();
            let identity = heartbeat_writer.get_identity();
            let sal_index = heartbeat_writer.get_index();
            loop {
                let seq_num = heartbeat_writer.get_seq_num();

                let heartbeat_topic = Heartbeat::default()
                    .with_timestamps()
                    .with_sal_index(sal_index)
                    .with_private_origin(origin)
                    .with_private_identity(&identity)
                    .with_private_seq_num(seq_num);
                let write_res = heartbeat_writer
                    .write_typed::<Heartbeat>(&heartbeat_topic)
                    .await;
                if write_res.is_err() {
                    log::error!("Failed to write heartbeat data {write_res:?}.");
                    break;
                }
                sleep(HEARTBEAT_TIME).await;
            }
        });
        self.heartbeat_task = Some(heartbeat_task);

        let controller_command_ack = ControllerCommandAck::start(&self.domain, &sal_info).await;

        for command in sal_info.get_command_names() {
            let controller_command_ack_sender = controller_command_ack.ack_sender.clone();
            log::debug!("Registering command {command}.");
            let command_sender = self.command_sender.clone();
            let mut controller_command =
                ControllerCommand::new(&command, &self.domain, &sal_info).unwrap();

            task::spawn(async move {
                loop {
                    if let Ok(command_data) = controller_command.process_command().await {
                        log::debug!("Received {command} with: {command_data:?}");
                        let ack_sender = controller_command_ack_sender.clone();
                        let _ = command_sender
                            .send((
                                CmdData {
                                    name: command.to_string(),
                                    data: command_data,
                                },
                                ack_sender,
                            ))
                            .await;
                    }
                }
            });
        }

        self.controller_command_ack = Some(controller_command_ack);
    }

    /// This method runs the control loop of the CSC.
    ///
    /// Once awaited the CSC will start to respond to commands.
    pub async fn run(&mut self) -> SalObjResult<()> {
        while let Some((data, ack_channel)) = self.command_receiver.recv().await {
            handle_command!(
                "start",
                "standby",
                "enable",
                "disable",
                "setScalars",
                "setArrays",
                "fault",
                "wait"
            );
        }
        Ok(())
    }

    /// Respond to the exitControl command.
    ///
    /// If the CSC is in Standby, this will terminate the CSC execution.
    async fn do_exit_control(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        let exit_control = from_value::<ExitControl>(&data.data).unwrap();
        let current_state = self.get_current_state();
        if current_state != State::Standby {
            return Ok((
                CommandAck::make_failed(
                    exit_control,
                    1,
                    &format!("Invalid state transition {current_state:?} -> Offline."),
                ),
                ack_channel,
            ));
        }
        self.set_summary_state(State::Offline);
        self.update_summary_state().await?;
        Ok((CommandAck::make_complete(exit_control), ack_channel))
    }

    /// Respond to the start command.
    ///
    /// This will transition the CSC from Standby to Disabled.
    async fn do_start(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        log::info!("do_start received {:?}", data.name);
        let start = from_value::<Start>(&data.data).unwrap();
        let current_state = self.get_current_state();
        if current_state != State::Standby {
            return Ok((
                CommandAck::make_failed(
                    start,
                    1,
                    &format!("Invalid state transition {current_state:?} -> Disable."),
                ),
                ack_channel,
            ));
        }
        let _ = self.configure(&start);

        let sal_info = SalInfo::new("Test", self.index).unwrap();

        let mut telemetry_writers: WriteTopicSet = sal_info
            .get_telemetry_names()
            .into_iter()
            .map(|telemetry_name| {
                (
                    telemetry_name.to_owned(),
                    WriteTopic::new(&telemetry_name, &sal_info, &self.domain),
                )
            })
            .collect();

        let mut telemetry_received = self.telemetry_receiver.clone();

        let telemetry_loop_task = task::spawn(async move {
            log::debug!("Telemetry task starting");

            let mut telemetry_data: HashMap<String, TestTelemetry> = HashMap::from([
                (
                    "scalars".to_owned(),
                    TestTelemetry::Scalars(Scalars::default()),
                ),
                (
                    "arrays".to_owned(),
                    TestTelemetry::Arrays(Arrays::default()),
                ),
            ]);

            loop {
                let loop_time_task = task::spawn(async { sleep(Duration::from_secs(1)).await });

                if timeout(Duration::from_secs(1), telemetry_received.changed())
                    .await
                    .is_ok()
                {
                    let new_telemetry = telemetry_received.borrow();
                    log::debug!("Updating telemetry data for {}", new_telemetry.name);
                    *telemetry_data
                        .entry(new_telemetry.name.to_owned())
                        .or_insert(new_telemetry.data.clone()) = new_telemetry.data.clone();
                } else {
                    log::trace!("Telemetry not updated.");
                }

                for (telemetry_name, telemetry_writer) in telemetry_writers.iter_mut() {
                    let name = telemetry_name.as_str();
                    if let Some(telemetry_data_to_write) = telemetry_data.get_mut(name) {
                        match telemetry_data_to_write {
                            TestTelemetry::Scalars(scalar) => {
                                let _ = telemetry_writer.write_typed::<Scalars>(scalar).await;
                            }
                            TestTelemetry::Arrays(array) => {
                                let _ = telemetry_writer.write_typed::<Arrays>(array).await;
                            }
                            TestTelemetry::None => {}
                        }
                    }
                }
                let _ = loop_time_task.await;
            }
        });
        self.telemetry_loop_task = Some(telemetry_loop_task);

        self.set_summary_state(State::Disabled);
        self.update_summary_state().await?;
        Ok((CommandAck::make_complete(start), ack_channel))
    }

    /// Respond to the standby command.
    ///
    /// This command will transition the CSC from Fault or Disabled into
    /// Standby.
    async fn do_standby(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        log::info!("do_standby received {:?}", data.name);
        let standby = from_value::<Standby>(&data.data).unwrap();
        let current_state = self.get_current_state();
        if !HashSet::from([State::Fault, State::Disabled]).contains(&current_state) {
            return Ok((
                CommandAck::make_failed(
                    standby,
                    1,
                    &format!("Invalid state transition {current_state:?} -> Standby."),
                ),
                ack_channel,
            ));
        }
        self.set_summary_state(State::Standby);
        self.update_summary_state().await?;
        Ok((CommandAck::make_complete(standby), ack_channel))
    }

    /// Respond to the enable command.
    ///
    /// This command will transition the CSC from Disabled to Enabled.
    async fn do_enable(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        log::info!("do_enable received {:?}", data.name);
        let enable = from_value::<Enable>(&data.data).unwrap();
        let current_state = self.get_current_state();
        if current_state != State::Disabled {
            return Ok((
                CommandAck::make_failed(
                    enable,
                    1,
                    &format!("Invalid state transition {current_state:?} -> Enabled."),
                ),
                ack_channel,
            ));
        }
        self.set_summary_state(State::Enabled);
        self.update_summary_state().await?;

        Ok((CommandAck::make_complete(enable), ack_channel))
    }

    /// Respond to the disable command.
    ///
    /// This command will transition the CSC from Enabled to Disabled.
    async fn do_disable(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        log::info!("do_disabled received {:?}", data.name);
        let disable = from_value::<Disable>(&data.data).unwrap();
        let current_state = self.get_current_state();
        if current_state != State::Enabled {
            return Ok((
                CommandAck::make_failed(
                    disable,
                    1,
                    &format!("Invalid state transition {current_state:?} -> Disable."),
                ),
                ack_channel,
            ));
        }
        self.set_summary_state(State::Disabled);
        self.update_summary_state().await?;
        if let Some(telemetry_loop_task) = &self.telemetry_loop_task {
            log::debug!("Stopping telemetry task.");
            telemetry_loop_task.abort();
        }
        Ok((CommandAck::make_complete(disable), ack_channel))
    }

    /// Respond to the setScalars command.
    ///
    /// This command is only valid when the CSC is in Enabled state. It will
    /// publish an event with the values of scalars topics and update the
    /// scalars telemetry.
    async fn do_set_scalars(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        if let Ok(scalars) = from_value::<Scalars>(&data.data) {
            log::debug!("setScalars received: {scalars:?}");
            let current_state = self.get_current_state();
            if current_state != State::Enabled {
                log::debug!("Invalid, current state {current_state}.");
                return Ok((
                    CommandAck::make_failed(
                        scalars,
                        1,
                        &format!("Command setScalars not allowed in {current_state:?}."),
                    ),
                    ack_channel,
                ));
            }
            match self
                .controller
                .write_event("logevent_scalars", &scalars)
                .await
            {
                Ok(_) => {
                    log::debug!("setScalars sent event, updating telemetry.");
                    let _ = self.telemetry_sender.send(TelemetryPayload {
                        name: "scalars".to_owned(),
                        data: TestTelemetry::Scalars(scalars.clone()),
                    });
                    log::debug!("setScalars telemetry sent, command completed.");

                    Ok((CommandAck::make_complete(scalars), ack_channel))
                }
                Err(error) => {
                    log::debug!("setScalars command failed.");
                    Ok((
                        CommandAck::make_failed(
                            scalars,
                            1,
                            &format!("Failed to parse event data: {error}."),
                        ),
                        ack_channel,
                    ))
                }
            }
        } else {
            log::error!("Cannot parse data.");
            Err(SalObjError::new("Cannot parse data."))
        }
    }

    /// Respond to a setArrays command.
    ///
    /// This is similar to the setScalars command but for arrays instead.
    async fn do_set_arrays(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        match from_value::<Arrays>(&data.data) {
            Ok(arrays) => {
                let current_state = self.get_current_state();
                if current_state != State::Enabled {
                    return Ok((
                        CommandAck::make_failed(
                            arrays,
                            1,
                            &format!("Command setArrays not allowed in {current_state:?}."),
                        ),
                        ack_channel,
                    ));
                }
                let original_arrays = arrays.clone();
                if self
                    .controller
                    .write_event("logevent_arrays", &arrays)
                    .await
                    .is_ok()
                {
                    let _ = self.telemetry_sender.send(TelemetryPayload {
                        name: "arrays".to_owned(),
                        data: TestTelemetry::Arrays(arrays.clone()),
                    });
                    Ok((CommandAck::make_complete(original_arrays), ack_channel))
                } else {
                    Ok((
                        CommandAck::make_failed(original_arrays, 1, "Failed to parse event data"),
                        ack_channel,
                    ))
                }
            }
            Err(error) => {
                let error_message = format!("Cannot parse data: {error}");
                log::error!("{error_message}");
                Err(SalObjError::new(&error_message))
            }
        }
    }

    /// Respond to the fault command.
    ///
    /// This will send the CSC to Fault.
    async fn do_fault(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        match from_value::<EmptyTopic>(&data.data) {
            Ok(fault) => {
                self.set_summary_state(State::Fault);
                self.update_summary_state().await?;
                Ok((CommandAck::make_complete(fault), ack_channel))
            }
            Err(error) => {
                let error_message = format!("Cannot parse data: {error}");
                log::error!("{error_message}");
                Err(SalObjError::new(&error_message))
            }
        }
    }

    /// Respond to the wait command.
    ///
    /// This command will wait for the specified duration before completing.
    /// If many commands are sent at the same time they are all executed in
    /// parallel.
    async fn do_wait(
        &mut self,
        data: &CmdData,
        ack_channel: mpsc::Sender<CommandAck>,
    ) -> SalObjResult<CommandAckResult> {
        match from_value::<Wait>(&data.data) {
            Ok(wait) => {
                let current_state = self.get_current_state();
                if current_state != State::Enabled {
                    return Ok((
                        CommandAck::make_failed(
                            wait,
                            1,
                            &format!("Command wait not allowed in {current_state:?}."),
                        ),
                        ack_channel,
                    ));
                }

                let timeout = Duration::from_secs((wait.duration * 2.0) as u64);

                let wait_data = wait.clone();
                let ack_channel_process = ack_channel.clone();
                // Ignore the return value here, just want to run this in the background
                // without waiting. The result is really not important.
                task::spawn(async move {
                    TestCSC::wait_and_ack(wait_data, ack_channel_process).await;
                });

                Ok((
                    CommandAck::make_in_progress(wait, timeout, "Wait command in progress."),
                    ack_channel,
                ))
            }
            Err(error) => {
                let error_message = format!("Cannot parse data: {error}");
                log::error!("{error_message}");
                Err(SalObjError::new(&error_message))
            }
        }
    }

    /// Publish the current state of the component.
    async fn update_summary_state(&mut self) -> SalObjResult<()> {
        let summary_state = self
            .controller
            .get_event_to_write::<SummaryState>("logevent_summaryState")?
            .with_summary_state(self.summary_state);
        // let mut summary_state = SummaryState::default();
        // summary_state.set_summary_state(self.summary_state);

        if let Err(err) = self
            .controller
            .write_event("logevent_summaryState", &summary_state)
            .await
        {
            return Err(SalObjError::new(&format!(
                "Failed to write summary state: {err:?}"
            )));
        }
        Ok(())
    }

    /// A task that will wait for a specified duration and then acknowledge
    /// a command.
    ///
    /// This is used by the TestCSC::do_wait method to implement the command response.
    async fn wait_and_ack(wait: Wait, ack_channel: mpsc::Sender<CommandAck>) {
        let wait_time = Duration::from_secs(wait.duration as u64);
        sleep(wait_time).await;
        let _ = ack_channel.send(CommandAck::make_complete(wait)).await;
    }
}

impl BaseCSC for TestCSC<'_> {
    fn get_current_state(&self) -> State {
        self.summary_state
    }

    fn set_summary_state(&mut self, new_state: State) {
        self.summary_state = new_state;
    }

    fn configure(&mut self, data: &Start) -> SalObjResult<()> {
        log::info!(
            "Received {} configuration override.",
            data.get_configuration_override()
        );
        Ok(())
    }
}
