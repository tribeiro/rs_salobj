use apache_avro::{from_value, to_value, types::Value};
use salobj::{
    csc::test_csc::{
        test_csc::TestCSC,
        topics::{arrays::Arrays, scalars::Scalars, wait::Wait},
    },
    domain::Domain,
    generics::summary_state::SummaryState,
    remote::Remote,
    sal_enums::{SalRetCode, State},
    topics::{base_sal_topic::BaseSALTopic, base_topic::BaseTopic, write_topic::WriteTopic},
};
use std::time::Duration;
use tokio::task;

macro_rules! process_test_csc_commands {
    ($cmd:expr, $record:ident) => {
        if $cmd == "command_setScalars" {
            let mut scalars = Scalars::default();
            scalars.set_sal_index(123);

            if let Ok(data_value) = to_value(scalars) {
                if let Value::Record(data_record) = data_value {
                    for (field, value) in data_record.into_iter() {
                        $record.put(&field, value);
                    }
                }
            }
        } else if $cmd == "command_setArrays" {
            let mut array = Arrays::default();
            array.set_sal_index(123);
            if let Ok(data_value) = to_value(array) {
                if let Value::Record(data_record) = data_value {
                    for (field, value) in data_record.into_iter() {
                        $record.put(&field, value);
                    }
                }
            }
        } else if $cmd == "command_wait" {
            let mut wait = Wait::default();
            wait.set_sal_index(123);
            if let Ok(data_value) = to_value(wait) {
                if let Value::Record(data_record) = data_value {
                    for (field, value) in data_record.into_iter() {
                        $record.put(&field, value);
                    }
                }
            }
        }
    };
}

#[tokio::test]
async fn test_state_transition() {
    let mut test_csc = TestCSC::new(123).unwrap();

    test_csc.start().await;

    let _ = task::spawn(async move {
        println!("Running CSC.");
        let _ = test_csc.run().await;
    });

    let mut domain = Domain::new();
    let mut remote = Remote::from_name_index(&mut domain, "Test", 123).unwrap();

    let timeout = Duration::from_secs(10);

    // Check that the initial state is Standby
    if let Ok(Some(summary_state)) = remote
        .pop_event_back("logevent_summaryState", false, timeout)
        .await
    {
        let summary_state = from_value::<SummaryState>(&summary_state)
            .unwrap()
            .get_summary_state();
        assert_eq!(summary_state, State::Standby);
    } else {
        panic!("Could not get initial summary state");
    }

    // Check that all commands that should be rejected while in standby are
    // rejected.
    let standby_test_commands = ["setScalars", "setArrays", "wait"];

    for cmd_name in standby_test_commands {
        println!("Testing {cmd_name}");
        let cmd = format!("command_{cmd_name}");

        let schema = remote.get_command_schema(&cmd).unwrap();
        let mut record = WriteTopic::make_data_type(&schema).unwrap();

        process_test_csc_commands!(cmd, record);

        if let Err(ack_cmd) = remote
            .run_command(cmd.to_string(), &mut record, timeout, true)
            .await
        {
            println!("{ack_cmd:?}");
            assert_eq!(*ack_cmd.get_ack_enum(), SalRetCode::CmdFailed);
            assert_eq!(ack_cmd.get_error(), 1);

            assert_eq!(
                ack_cmd.get_result(),
                format!("Command {cmd_name} not allowed in Standby.")
            );
        } else {
            panic!("Command {cmd} should fail.");
        }
    }

    // Send the CSC to Disabled and test the same thing.
    let cmd = "command_start";
    let schema = remote.get_command_schema(cmd).unwrap();
    let mut record = WriteTopic::make_data_type(&schema).unwrap();
    record.put("configurationOverride", Value::String("".to_owned()));

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

    for cmd_name in standby_test_commands {
        println!("Testing {cmd_name}");
        let cmd = format!("command_{cmd_name}");

        let schema = remote.get_command_schema(&cmd).unwrap();
        let mut record = WriteTopic::make_data_type(&schema).unwrap();

        process_test_csc_commands!(cmd, record);

        if let Err(ack_cmd) = remote
            .run_command(cmd.to_string(), &mut record, timeout, true)
            .await
        {
            println!("{ack_cmd:?}");
            assert_eq!(*ack_cmd.get_ack_enum(), SalRetCode::CmdFailed);
            assert_eq!(ack_cmd.get_error(), 1);

            assert_eq!(
                ack_cmd.get_result(),
                format!("Command {cmd_name} not allowed in Disabled.")
            );
        } else {
            panic!("Command {cmd} should fail.");
        }
    }
}
