use crate::{topics::base_sal_topic::BaseSALTopic, utils::xml_utils::get_default_sal_index};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};

#[add_sal_topic_fields]
#[derive(Debug, Default, Deserialize, Serialize, BaseSALTopic)]
pub struct AckCmd {
    /// Acknowledgement code.
    ack: i32,
    /// Error number.
    error: i32,
    /// Result string with an explanation for the ack.
    result: String,
    /// Identity of the component that issued the command.
    identity: String,
    /// origin of the component that issued the command.
    origin: i32,
    /// Index of command in alphabetical list of commands, with 0 being the first.
    cmdtype: i32,
    /// Estimated timeout for the command.
    timeout: f64,
}

impl AckCmd {
    pub fn new(
        ack: i32,
        error: i32,
        result: &str,
        identity: &str,
        origin: i32,
        cmdtype: i32,
        timeout: f64,
    ) -> AckCmd {
        AckCmd {
            ack,
            error,
            result: result.to_owned(),
            identity: identity.to_owned(),
            origin,
            cmdtype,
            timeout,
            ..Default::default()
        }
    }

    pub fn get_cmdtype(&self) -> i32 {
        self.cmdtype
    }
}
