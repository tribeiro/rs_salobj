//! Handles reading command topic and writing acknowledgements.

use apache_avro::types::Value;
use log;
use std::time::Instant;

use crate::{
    domain::Domain,
    error::errors::{SalObjError, SalObjResult},
    sal_info::SalInfo,
    topics::{base_sal_topic::BaseSALTopic, read_topic::ReadTopic, write_topic::WriteTopic},
    utils::{command_ack::CommandAck, types::WriteTopicResult},
};

pub struct ControllerCommand<'a> {
    command_name: String,
    command_reader: ReadTopic<'a>,
    ack_writer: WriteTopic<'a>,
    command_type: usize,
}

impl<'a> ControllerCommand<'a> {
    pub fn new(
        command_name: &str,
        domain: &Domain,
        sal_info: &SalInfo,
    ) -> SalObjResult<ControllerCommand<'a>> {
        if let Some(command_type) = sal_info.get_command_type(command_name) {
            Ok(ControllerCommand {
                command_name: command_name.to_owned(),
                command_reader: ReadTopic::new(command_name, sal_info, domain, 0),
                ack_writer: WriteTopic::new("ackcmd", sal_info, domain),
                command_type,
            })
        } else {
            Err(SalObjError::new(&format!(
                "Could not find command type for {command_name}"
            )))
        }
    }

    pub fn get_command_type(&self) -> i64 {
        self.command_type as i64
    }

    pub async fn process_command(&mut self) -> SalObjResult<Value> {
        let start = Instant::now();

        log::trace!("process_command {} start", self.command_name);
        if let Some(cmd_data) = self
            .command_reader
            .pop_front(false, std::time::Duration::from_millis(100))
            .await
        {
            let duration = start.elapsed();
            log::trace!(
                "process_command {} finished took {duration:?} to take data.",
                self.command_name
            );
            Ok(cmd_data)
        } else {
            log::trace!("process_command {} finished no data.", self.command_name);
            Err(SalObjError::new("No command received."))
        }
    }

    pub async fn ack<'si>(&mut self, command_ack: CommandAck) -> WriteTopicResult {
        let mut ackcmd = command_ack.to_ackcmd();
        ackcmd.set_private_seq_num(command_ack.get_seq_num());
        self.ack_writer.write_typed(&mut ackcmd).await
    }
}
