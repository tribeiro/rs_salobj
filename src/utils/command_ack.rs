use std::fmt;

use crate::sal_enums::{self, SalRetCode};
use crate::topics::base_sal_topic::BaseSALTopic;

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

    pub fn make_in_progress<T>(cmd: T, timeout: std::time::Duration, result: &str) -> CommandAck
    where
        T: BaseSALTopic,
    {
        CommandAck {
            ack: SalRetCode::CmdInprogress,
            error: 0,
            result: result.to_owned(),
            identity: cmd.get_private_identity().to_owned(),
            origin: cmd.get_private_origin(),
            timeout,
            seq_num: cmd.get_private_seq_num(),
        }
    }

    pub fn make_complete<T>(cmd: T) -> CommandAck
    where
        T: BaseSALTopic,
    {
        CommandAck {
            ack: SalRetCode::CmdComplete,
            error: 0,
            result: "".to_owned(),
            identity: cmd.get_private_identity().to_owned(),
            origin: cmd.get_private_origin(),
            timeout: std::time::Duration::new(0, 0),
            seq_num: cmd.get_private_seq_num(),
        }
    }

    pub fn make_failed<T>(cmd: T, error: isize, result: &str) -> CommandAck
    where
        T: BaseSALTopic,
    {
        CommandAck {
            ack: SalRetCode::CmdFailed,
            error,
            result: result.to_owned(),
            identity: cmd.get_private_identity().to_owned(),
            origin: cmd.get_private_origin(),
            timeout: std::time::Duration::new(0, 0),
            seq_num: cmd.get_private_seq_num(),
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

    pub fn get_ack(&self) -> i64 {
        self.ack.clone() as i64
    }

    pub fn get_error(&self) -> isize {
        self.error
    }

    pub fn get_result(&self) -> &str {
        &self.result
    }
}

#[cfg(test)]
mod tests {}
