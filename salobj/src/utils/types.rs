use apache_avro::types::Value;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

use crate::{
    error::errors::SalObjError,
    sal_subsystem::SALSubsystemInfo,
    topics::{
        controller_command::ControllerCommand, read_topic::ReadTopic,
        remote_command::RemoteCommand, write_topic::WriteTopic,
    },
};
use serde::{Deserialize, Serialize, Serializer};

pub type WriteTopicSet<'a> = HashMap<String, WriteTopic<'a>>;
pub type ReadTopicSet<'a> = HashMap<String, ReadTopic<'a>>;
pub type RemoteCommandSet<'a> = HashMap<String, RemoteCommand<'a>>;
pub type ControllerCommandSet<'a> = HashMap<String, ControllerCommand<'a>>;
pub type WriteTopicResult = Result<i32, SalObjError>;
pub type ControllerCallbackFunc = Option<Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = ()>>>>>;
pub type SALSubsystemInfoRet = Result<SALSubsystemInfo, Box<dyn Error>>;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum SalDouble {
    Some(f64),
    None,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum SalFloat {
    Some(f32),
    None,
}

impl Default for SalDouble {
    fn default() -> Self {
        SalDouble::Some(0.0)
    }
}

impl Default for SalFloat {
    fn default() -> Self {
        SalFloat::Some(0.0)
    }
}

impl Serialize for SalDouble {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SalDouble::Some(value) => serializer.serialize_f64(*value),
            SalDouble::None => serializer.serialize_unit(),
        }
    }
}

impl Serialize for SalFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            SalFloat::Some(value) => serializer.serialize_f32(*value),
            SalFloat::None => serializer.serialize_unit(),
        }
    }
}
