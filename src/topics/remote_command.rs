//! Handles Writing command topics and waiting for acknowledgement.
//!

use crate::{
    domain::Domain,
    sal_enums,
    sal_info::SalInfo,
    topics::{read_topic::ReadTopic, write_topic::WriteTopic},
    utils::command_ack::CommandAck,
};
use apache_avro::types::{Record, Value};
use std::{collections::HashMap, time::Duration};

pub type AckCmdResult = std::result::Result<CommandAck, CommandAck>;

pub struct RemoteCommand {
    command_writer: WriteTopic,
    ack_reader: ReadTopic,
}

impl RemoteCommand {
    pub fn new(command_name: &str, domain: &Domain, sal_info: &SalInfo) -> RemoteCommand {
        RemoteCommand {
            command_writer: WriteTopic::new(command_name, sal_info, domain),
            ack_reader: ReadTopic::new("ackcmd", sal_info, domain, 0),
        }
    }

    pub async fn run<'a, 'b>(
        &mut self,
        parameters: &mut Record<'b>,
        timeout: Duration,
        wait_done: bool,
        sal_info: &SalInfo<'a>,
    ) -> AckCmdResult {
        let identity = self.command_writer.get_identity();
        let origin = self.command_writer.get_origin();
        match self.command_writer.write(parameters, sal_info).await {
            Ok(seq_num) => loop {
                if let Some(Value::Record(ack_cmd)) =
                    self.ack_reader.pop_back(false, timeout, sal_info).await
                {
                    let data_dict: HashMap<String, Value> = ack_cmd
                        .into_iter()
                        .map(|(field, value)| (field, value))
                        .collect();

                    if *data_dict.get("origin").unwrap_or(&Value::Long(0)) == Value::Long(origin)
                        && *data_dict
                            .get("identity")
                            .unwrap_or(&Value::String("".to_owned()))
                            == Value::String(identity.to_owned())
                        && *data_dict.get("private_seqNum").unwrap_or(&Value::Long(0))
                            == Value::Long(seq_num)
                    {
                        let ack = sal_enums::get_ackcmd_code(data_dict.get("ack")).clone();
                        let error: isize = match data_dict.get("error") {
                            Some(Value::Int(error)) => *error as isize,
                            _ => 0,
                        };
                        let result = match data_dict.get("result") {
                            Some(Value::String(result)) => result.to_owned(),
                            _ => "".to_string(),
                        };

                        let command_ack = CommandAck::new(
                            ack,
                            error,
                            result,
                            identity.clone(),
                            origin,
                            timeout,
                            seq_num,
                        );

                        if !wait_done {
                            return Ok(command_ack);
                        } else if command_ack.is_final() {
                            if command_ack.is_good() {
                                return Ok(command_ack);
                            } else {
                                return Err(command_ack);
                            };
                        }
                    }
                } else {
                    return Err(CommandAck::new(
                        sal_enums::SalRetCode::CmdNoack,
                        -1,
                        "No acknowledgment seen.".to_string(),
                        identity,
                        origin,
                        timeout,
                        seq_num,
                    ));
                }
            },
            Err(error) => Err(CommandAck::invalid_command(&error.to_string())),
        }
    }
}
