//! Handles reading command topic and writing acknowledgements.

use apache_avro::types::Value;
use std::future::Future;
use std::pin::Pin;

use crate::{
    domain::Domain,
    error::errors::{SalObjError, SalObjResult},
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, read_topic::ReadTopic, write_topic::WriteTopic},
    utils::{command_ack::CommandAck, types::WriteTopicResult},
};

pub struct ControllerCommand {
    command_reader: ReadTopic,
    ack_writer: WriteTopic,
    command_type: usize,
    callback: Option<Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = ()>>>>>,
}

impl ControllerCommand {
    pub fn new(
        command_name: &str,
        domain: &Domain,
        sal_info: &SalInfo,
    ) -> SalObjResult<ControllerCommand> {
        if let Some(command_type) = sal_info.get_command_type(command_name) {
            Ok(ControllerCommand {
                command_reader: ReadTopic::new(command_name, sal_info, domain, 0),
                ack_writer: WriteTopic::new("ackcmd", sal_info, domain),
                command_type: command_type,
                callback: None,
            })
        } else {
            Err(SalObjError::new(&format!(
                "Could not find command type for {command_name}"
            )))
        }
    }

    fn get_command_type(&self) -> i64 {
        self.command_type as i64
    }

    pub fn set_callback(
        &mut self,
        callback: Option<Box<dyn Fn(Value) -> Pin<Box<dyn Future<Output = ()>>>>>,
    ) {
        self.callback = callback
    }

    pub async fn process_command<'si>(&mut self, sal_info: &SalInfo<'si>) -> SalObjResult<usize> {
        if let Some(cmd_data) = self
            .command_reader
            .pop_front(false, std::time::Duration::from_millis(100), sal_info)
            .await
        {
            if let Some(callback) = &self.callback {
                let future = callback(cmd_data);
                future.await;
                Ok(1)
            } else {
                println!("callback not set");
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    pub async fn ack<'si>(
        &mut self,
        ackcmd: CommandAck,
        sal_info: &SalInfo<'si>,
    ) -> WriteTopicResult {
        let schema = sal_info.get_topic_schema("ackcmd").unwrap().clone();
        let mut ackcmd_record = WriteTopic::make_data_type(&schema).unwrap();

        ackcmd_record.put("private_seqNum", Value::Long(ackcmd.get_seq_num()));
        ackcmd_record.put("origin", Value::Long(ackcmd.get_origin()));
        ackcmd_record.put("identity", Value::String(ackcmd.get_identity().to_owned()));
        ackcmd_record.put("cmdtype", Value::Long(self.get_command_type()));
        ackcmd_record.put("ack", Value::Long(ackcmd.get_ack()));
        ackcmd_record.put("error", Value::Long(ackcmd.get_error() as i64));
        ackcmd_record.put("result", Value::String(ackcmd.get_result().to_owned()));
        ackcmd_record.put("timeout", Value::Double(ackcmd.get_timeout().as_secs_f64()));

        self.ack_writer.write(&mut ackcmd_record, sal_info).await
    }
}
