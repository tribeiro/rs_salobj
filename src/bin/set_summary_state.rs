use apache_avro::from_value;
use apache_avro::types::Value;
use clap::Parser;
use salobj::{
    domain,
    generics::summary_state::SummaryState,
    remote::Remote,
    sal_enums::State,
    topics::{base_topic::BaseTopic, write_topic::WriteTopic},
    utils::{cli::LogLevel, csc::compute_state_transitions},
};
use simple_logger::SimpleLogger;
use std::time::Duration;

/// Put the CSC in a particular state.
#[derive(Parser)]
struct Cli {
    /// Component name
    #[clap(short = 'c', long = "component")]
    component: String,

    /// Component index
    #[clap(short = 'i', long = "index", default_value = "0")]
    index: isize,

    /// Desired state
    #[clap(short = 's', long = "state")]
    desired_state: State,

    /// Configuration override
    #[clap(long = "config", default_value = "")]
    configuration_override: String,

    #[arg(value_enum, long = "log-level", default_value_t = LogLevel::Info)]
    log_level: LogLevel,
}

impl Cli {
    pub fn get_component_name(&self) -> String {
        self.component.to_owned()
    }

    pub fn get_desired_state(&self) -> State {
        self.desired_state
    }

    pub fn get_configuration_override(&self) -> String {
        self.configuration_override.to_owned()
    }

    pub fn get_component_index(&self) -> isize {
        self.index
    }

    pub fn get_log_level(&self) -> &LogLevel {
        &self.log_level
    }
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let cli = Cli::parse();

    match cli.get_log_level() {
        LogLevel::Trace => log::set_max_level(log::LevelFilter::Trace),
        LogLevel::Debug => log::set_max_level(log::LevelFilter::Debug),
        LogLevel::Info => log::set_max_level(log::LevelFilter::Info),
        LogLevel::Warn => log::set_max_level(log::LevelFilter::Warn),
        LogLevel::Error => log::set_max_level(log::LevelFilter::Error),
    };

    log::info!(
        "Sending {} to {:?} state [config:{}].",
        cli.get_component_name(),
        cli.get_desired_state(),
        cli.get_configuration_override()
    );

    let mut domain = domain::Domain::new();
    let mut remote = Remote::from_name_index(
        &mut domain,
        &cli.get_component_name(),
        cli.get_component_index(),
    )
    .unwrap();

    let timeout = Duration::from_secs(10);
    let wait_done = true;

    log::debug!("Getting current state.");
    if let Ok(Some(summary_state)) = remote
        .pop_event_back("logevent_summaryState", false, timeout)
        .await
    {
        let summary_state = from_value::<SummaryState>(&summary_state)
            .unwrap()
            .get_summary_state();

        log::debug!("Current state: {summary_state:?}");

        if let Some(state_transition_commands) =
            compute_state_transitions(summary_state, cli.get_desired_state())
        {
            for cmd in state_transition_commands.iter() {
                log::debug!("Sending command: {cmd}");
                let schema = remote.get_command_schema(cmd).unwrap();
                let mut record = WriteTopic::make_data_type(&schema).unwrap();

                if cmd.contains("command_start") {
                    record.put(
                        "configurationOverride",
                        Value::String(cli.get_configuration_override()),
                    );
                }

                let ack = remote
                    .run_command(cmd.to_string(), &mut record, timeout, wait_done)
                    .await;

                match ack {
                    Ok(_) => log::info!("Command ok"),
                    Err(ack) => panic!("Command failed with {ack:?}"),
                }
            }
        } else {
            log::warn!("No state transitions.");
        }
    } else {
        log::error!("No summary state from component.");
    }
    log::info!("Done...");
}
