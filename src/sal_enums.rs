/// SAL return codes.
pub enum SalRetCode {
    Ok = 0,
    Error = -1,
    IllegalRevcode = -2,
    TooManyHandles = -3,
    NotDefined = -4,

    // Generic State machine states
    StateDisabled = 1,
    StateEnabled = 2,
    StateFault = 3,
    StateOffline = 4,
    StateStandby = 5,

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
