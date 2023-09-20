use std::collections::HashMap;

use crate::{
    error::errors::SalObjError,
    topics::{read_topic::ReadTopic, remote_command::RemoteCommand, write_topic::WriteTopic},
};

pub type WriteTopicSet = HashMap<String, WriteTopic>;
pub type ReadTopicSet = HashMap<String, ReadTopic>;
pub type RemoteCommandSet = HashMap<String, RemoteCommand>;

pub type WriteTopicResult = Result<i64, SalObjError>;
