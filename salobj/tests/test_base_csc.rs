use apache_avro::{from_value, types::Value};
use salobj::{
    csc::test_csc::{
        csc::TestCSC,
        topics::{arrays::Arrays, scalars::Scalars, wait::Wait},
    },
    domain::Domain,
    generics::summary_state::SummaryState,
    remote::Remote,
    sal_enums::{SalRetCode, State},
    topics::{base_topic::BaseTopic, write_topic::WriteTopic},
};
use simple_logger::SimpleLogger;
use std::time::Duration;
use tokio::task;

macro_rules! assert_command_fails {
    ($cmd:expr, $remote:ident, $current_state:ident) => {
        if $cmd == "command_setScalars" {
            // let mut scalars = Scalars::default().with_sal_index(123);
            let scalars: Scalars = $remote.get_command_data("command_setScalars").unwrap();
            process_command!($cmd, $remote, scalars, $current_state);
        } else if $cmd == "command_setArrays" {
            //let mut array = Arrays::default().with_sal_index(123);
            let array: Arrays = $remote.get_command_data("command_setArrays").unwrap();
            process_command!($cmd, $remote, array, $current_state);
        } else if $cmd == "command_wait" {
            // let mut wait = Wait::default().with_sal_index(123);
            let wait: Wait = $remote.get_command_data("command_wait").unwrap();
            process_command!($cmd, $remote, wait, $current_state);
        }
    };
}

macro_rules! process_command {
    ($cmd:expr, $remote:ident, $data:ident, $current_state:ident) => {
        if let Err(ack_cmd) = $remote
            .run_command_typed(&$cmd, &$data, Duration::from_secs(5), true)
            .await
        {
            log::debug!("{ack_cmd:?}");
            let cmd_name: Vec<&str> = $cmd.split("_").collect();

            assert_eq!(*ack_cmd.get_ack_enum(), SalRetCode::CmdFailed);
            assert_eq!(ack_cmd.get_error(), 1);

            assert_eq!(
                ack_cmd.get_result(),
                format!("Command {} not allowed in {}.", cmd_name[1], $current_state)
            );
        } else {
            panic!("Command {} should fail.", $cmd);
        }
    };
}

#[tokio::test]
async fn test_state_transition() {
    SimpleLogger::new().init().unwrap();

    log::set_max_level(log::LevelFilter::Debug);

    let mut test_csc = TestCSC::new(123).await.unwrap();

    test_csc.start().await;

    let run_csc_task = task::spawn(async move {
        println!("Running CSC.");
        let _ = test_csc.run().await;
    });

    let mut domain = Domain::new();
    let mut remote = Remote::from_name_index(&mut domain, "Test", 123)
        .await
        .unwrap();

    let timeout = Duration::from_secs(20);

    // Check that the initial state is Standby
    match remote
        .pop_event_back("logevent_summaryState", false, timeout)
        .await
    {
        Ok(Some(summary_state)) => {
            let summary_state = from_value::<SummaryState>(&summary_state)
                .unwrap()
                .get_summary_state();
            assert_eq!(summary_state, State::Standby);
        }
        Ok(None) => panic!("No summary state data!"),
        Err(error) => panic!("Could not get initial summary state: {error:?}"),
    };

    println!("Test commands are rejected if CSC is in Standby.");
    // Check that all commands that should be rejected while in standby are
    // rejected.
    let standby_test_commands = ["setScalars", "setArrays", "wait"];
    let current_state = "Standby";

    for cmd_name in standby_test_commands {
        println!("Testing {cmd_name}");
        let cmd = format!("command_{cmd_name}");
        assert_command_fails!(cmd, remote, current_state);
    }

    println!("Sending CSC to Disabled.");

    // Send the CSC to Disabled and test the same thing.
    let cmd = "command_start";
    let schema = remote.get_command_schema(cmd).unwrap();
    let mut record = WriteTopic::make_data_type(&schema).unwrap();
    record.put("configurationOverride", Value::String("".to_owned()));

    println!("Test commands are rejected if CSC is in Disabled.");

    if let Ok(ack_cmd) = remote
        .run_command(cmd.to_string(), &mut record, timeout, true)
        .await
    {
        println!("{ack_cmd:?}");
        assert_eq!(*ack_cmd.get_ack_enum(), SalRetCode::CmdComplete);
        assert_eq!(ack_cmd.get_error(), 0);
    } else {
        panic!("Command {cmd} should succeed.");
    }

    let current_state = "Disabled";
    for cmd_name in standby_test_commands {
        println!("Testing {cmd_name}");
        let cmd = format!("command_{cmd_name}");

        assert_command_fails!(cmd, remote, current_state);
    }
    run_csc_task.abort();
}
