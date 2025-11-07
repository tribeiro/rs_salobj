use serde::Deserialize;
use ts_xml_define::ts_xml_define;

ts_xml_define!(Test);

#[test]
fn test_telemetry_scalars() {
    let scalars = TestScalars::default();
    assert_eq!(scalars.int0, 0);
}

#[test]
fn test_telemetry_arrays() {
    let arrays = TestArrays::default();
    assert_eq!(arrays.int0, [0, 0, 0, 0, 0]);
}

#[test]
fn test_logevent_arrays() {
    let arrays = TestLogeventArrays::default();
    assert_eq!(arrays.int0, [0, 0, 0, 0, 0]);
}

#[test]
fn test_command_set_scalars() {
    let set_scalars = TestCommandSetScalars::default();
    assert_eq!(set_scalars.int0, 0);
}

#[test]
fn test_command_set_arrays() {
    let arrays = TestCommandSetArrays::default();
    assert_eq!(arrays.int0, [0, 0, 0, 0, 0]);
}

#[test]
fn test_command_fault() {
    let fault = TestCommandFault::default();
    assert_eq!(fault.private_sndStamp, 0.);
}

#[test]
fn test_generic_command_start() {
    let topic = TestCommandStart::default();
    assert_eq!(topic.configurationOverride, "");
}

#[test]
fn test_generic_command_enable() {
    let topic = TestCommandEnable::default();
    assert_eq!(topic.private_sndStamp, 0.);
}

#[test]
fn test_generic_command_disable() {
    let topic = TestCommandDisable::default();
    assert_eq!(topic.private_sndStamp, 0.);
}

#[test]
fn test_generic_command_standby() {
    let topic = TestCommandStandby::default();
    assert_eq!(topic.private_sndStamp, 0.);
}

#[test]
fn test_generic_command_enter_control() {
    let topic = TestCommandEnterControl::default();
    assert_eq!(topic.private_sndStamp, 0.);
}

#[test]
fn test_generic_command_exit_control() {
    let topic = TestCommandExitControl::default();
    assert_eq!(topic.private_sndStamp, 0.);
}

#[test]
fn test_generic_command_abort() {
    let topic = TestCommandAbort::default();
    assert_eq!(topic.private_sndStamp, 0.);
}

#[test]
fn test_generic_command_set_log_level() {
    let topic = TestCommandSetLogLevel::default();
    assert_eq!(topic.level, 0);
}

#[test]
fn test_generic_logevent_summary_state() {
    let topic = TestLogeventSummaryState::default();
    assert_eq!(topic.summaryState, 0);
}

#[test]
fn test_generic_logevent_heartbeat() {
    let topic = TestLogeventHeartbeat::default();
    assert!(!topic.heartbeat);
}
