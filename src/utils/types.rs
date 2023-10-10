use std::collections::HashMap;
use apache_avro::types::Value;
use std::pin::Pin;
use std::future::Future;

use crate::{
    error::errors::SalObjError,
    topics::{
        controller_command::ControllerCommand, read_topic::ReadTopic,
        remote_command::RemoteCommand, write_topic::WriteTopic,
    },
};

pub type WriteTopicSet = HashMap<String, WriteTopic>;
pub type ReadTopicSet = HashMap<String, ReadTopic>;
pub type RemoteCommandSet = HashMap<String, RemoteCommand>;
pub type ControllerCommandSet = HashMap<String, ControllerCommand>;
pub type WriteTopicResult = Result<i64, SalObjError>;
pub type ControllerCallbackFunc = Option<Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = ()>>>>>;
