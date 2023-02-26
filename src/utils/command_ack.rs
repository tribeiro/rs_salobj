use std::fmt;

use crate::sal_enums::{self, SalRetCode};

#[derive(Debug, Clone)]
pub struct CommandAck {
    /// Acknowledgement code.
    ack: SalRetCode,
    /// Error number.
    error: isize,
    /// Result string with an explanation for the ack.
    result: String,
    /// Identity of the component that issued the command.
    identity: String,
    /// origin of the component that issued the command.
    origin: i64,
    /// Estimated timeout for the command.
    timeout: std::time::Duration,
    /// Sequence number of the command issued.
    seq_num: i64,
}

impl fmt::Display for CommandAck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?}][error:{}]::{}", self.ack, self.error, self.result)
    }
}

impl Default for CommandAck {
    fn default() -> Self {
        Self {
            ack: SalRetCode::CmdAck,
            error: 0,
            result: "".to_owned(),
            identity: "".to_owned(),
            origin: 0,
            timeout: std::time::Duration::from_secs(0),
            seq_num: 0,
        }
    }
}

impl CommandAck {
    pub fn new(
        ack: SalRetCode,
        error: isize,
        result: String,
        identity: String,
        origin: i64,
        timeout: std::time::Duration,
        seq_num: i64,
    ) -> CommandAck {
        CommandAck {
            ack,
            error,
            result,
            identity,
            origin,
            timeout,
            seq_num,
        }
    }

    pub fn invalid_command(result: &str) -> CommandAck {
        CommandAck {
            result: result.to_owned(),
            ..Default::default()
        }
    }
    /// Is the acknowledgement final?
    ///
    /// No more acks should be expected after this.
    pub fn is_final(&self) -> bool {
        sal_enums::is_ack_final(&self.ack)
    }

    /// Is this acknowledgement a good/non failed response?
    pub fn is_good(&self) -> bool {
        sal_enums::is_ack_good(&self.ack)
    }

    pub fn get_identity(&self) -> &str {
        &self.identity
    }

    pub fn get_origin(&self) -> i64 {
        self.origin
    }

    pub fn get_timeout(&self) -> std::time::Duration {
        self.timeout
    }

    pub fn get_seq_num(&self) -> i64 {
        self.seq_num
    }
}

#[cfg(test)]
mod tests {}
