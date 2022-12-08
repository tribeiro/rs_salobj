use apache_avro::from_value;
use apache_avro::types::Value;
use clap::Parser;
use salobj::{
    domain,
    generics::summary_state::SummaryState,
    remote::Remote,
    sal_enums::State,
    topics::{base_topic::BaseTopic, write_topic::WriteTopic},
    utils::csc::compute_state_transitions,
};
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!(
        "Sending {} to {:?} state [config:{}].",
        cli.get_component_name(),
        cli.get_desired_state(),
        cli.get_configuration_override()
    );

    let domain = domain::Domain::new();
    let mut remote = Remote::from_name_index(
        &domain,
        &cli.get_component_name(),
        cli.get_component_index(),
    );

    let timeout = Duration::from_secs(10);
    let wait_done = true;

    println!("Getting current state.");
    if let Ok(Some(summary_state)) = remote
        .pop_event_back("logevent_summaryState", false, timeout)
        .await
    {
        let summary_state = from_value::<SummaryState>(&summary_state)
            .unwrap()
            .get_summary_state();

        println!("Current state: {summary_state:?}");

        let state_transition_commands =
            compute_state_transitions(summary_state, cli.get_desired_state());
        for cmd in state_transition_commands.iter() {
            println!("Sending command: {cmd}");
            let schema = remote.get_command_schema(cmd);
            let mut record = WriteTopic::make_data_type(&schema);

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
                Ok(_) => println!("Command ok"),
                Err(ack) => panic!("Command failed with {ack:?}"),
            }
        }
    } else {
        panic!("No summary state from component.");
    }
    println!("Done...");
}
