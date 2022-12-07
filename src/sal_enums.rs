/// SAL return codes.
#[derive(Debug, PartialEq, Clone)]
pub enum SalRetCode {
    Ok = 0,
    Error = -1,
    IllegalRevcode = -2,
    TooManyHandles = -3,
    NotDefined = -4,

    // Timeout return codes
    Timeout = -5,
    SignalInterrupt = -6,

    // getSample timeout specifiers (+ve is a time in microseconds)
    WaitForNextUpdate = -10000,
    WaitForChange = -10001,

    // telemetry stream update types
    NoUpdates = -100,
    WaitingForNext = 100,
    GotUpdate = 101,
    SyncIn = 102,
    SyncOut = 103,
    SyncSet = 104,
    SyncClear = 105,
    SyncRead = 106,

    // generateAlert types
    EventInfo = 200,
    EventWarn = -200,
    EventError = -201,
    EventAbort = -202,

    // issueCommand/getResponse return codes
    CmdAck = 300,
    CmdInprogress = 301,
    CmdStalled = 302,
    CmdComplete = 303,
    CmdNoperm = -300,
    CmdNoack = -301,
    CmdFailed = -302,
    CmdAborted = -303,
    CmdTimeout = -304,

    // callback types for subscriptions
    DataAvail = 400,
    DeadlineMiss = 401,
    IncompatQos = 402,
    SampleRej = 403,
    LivelinessChg = 404,
    SampleLost = 405,
    SubscrMatch = 406,
}

/// Convert a Value::Long into a SalRetCode enum.
pub fn get_ackcmd_code(ackcmd: &Value) -> SalRetCode {
    match ackcmd {
        Value::Long(300) => SalRetCode::CmdAck,
        Value::Long(301) => SalRetCode::CmdInprogress,
        Value::Long(302) => SalRetCode::CmdStalled,
        Value::Long(303) => SalRetCode::CmdComplete,
        Value::Long(-300) => SalRetCode::CmdNoperm,
        Value::Long(-301) => SalRetCode::CmdNoack,
        Value::Long(-302) => SalRetCode::CmdFailed,
        Value::Long(-303) => SalRetCode::CmdAborted,
        Value::Long(-304) => SalRetCode::CmdTimeout,
        _ => SalRetCode::CmdAck,
    }
}

/// Is the ack final?
pub fn is_ack_final(ack: &SalRetCode) -> bool {
    [
        SalRetCode::CmdAborted,
        SalRetCode::CmdComplete,
        SalRetCode::CmdFailed,
        SalRetCode::CmdNoack,
        SalRetCode::CmdNoperm,
        SalRetCode::CmdStalled,
        SalRetCode::CmdTimeout,
    ]
    .contains(ack)
}

/// Is the ack good?
pub fn is_ack_good(ack: &SalRetCode) -> bool {
    [
        SalRetCode::CmdAck,
        SalRetCode::CmdInprogress,
        SalRetCode::CmdComplete,
    ]
    .contains(ack)
}

/// Standard state enumeration.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum State {
    // Generic State machine states
    Invalid,
    Disabled = 1,
    Enabled = 2,
    Fault = 3,
    Offline = 4,
    Standby = 5,
}

impl FromStr for State {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Disabled" => Ok(State::Disabled),
            "Enabled" => Ok(State::Enabled),
            "Fault" => Ok(State::Fault),
            "Offline" => Ok(State::Offline),
            "Standby" => Ok(State::Standby),
            _ => Ok(State::Invalid),
        }
    }
}

impl fmt::Display for State {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{:?}", self)
    }
}

