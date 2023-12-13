use apache_avro::types::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::{
    error::errors::SalObjError,
    topics::{
        controller_command::ControllerCommand, read_topic::ReadTopic,
        remote_command::RemoteCommand, write_topic::WriteTopic,
    },
};

pub type WriteTopicSet<'a> = HashMap<String, WriteTopic<'a>>;
pub type ReadTopicSet<'a> = HashMap<String, ReadTopic<'a>>;
pub type RemoteCommandSet<'a> = HashMap<String, RemoteCommand<'a>>;
pub type ControllerCommandSet<'a> = HashMap<String, ControllerCommand<'a>>;
pub type WriteTopicResult = Result<i64, SalObjError>;
pub type ControllerCallbackFunc = Option<Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = ()>>>>>;
