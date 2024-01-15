//! Handles Writing command topics and waiting for acknowledgement.
//!

use crate::{
    domain::Domain,
    sal_enums,
    sal_info::SalInfo,
    topics::{base_sal_topic::BaseSALTopic, read_topic::ReadTopic, write_topic::WriteTopic},
    utils::command_ack::CommandAck,
};
use apache_avro::{
    types::{Record, Value},
    Schema,
};
use serde::Serialize;
use std::fmt::Debug;
use std::{collections::HashMap, time::Duration};

pub type AckCmdResult = std::result::Result<CommandAck, CommandAck>;

pub struct RemoteCommand<'a> {
    command_writer: WriteTopic<'a>,
    ack_reader: ReadTopic<'a>,
}

impl<'a> RemoteCommand<'a> {
    pub fn new(command_name: &str, domain: &Domain, sal_info: &SalInfo) -> RemoteCommand<'a> {
        RemoteCommand {
            command_writer: WriteTopic::new(command_name, sal_info, domain),
            ack_reader: ReadTopic::new("ackcmd", sal_info, domain, 0),
        }
    }

    pub fn get_schema(&self) -> &Schema {
        self.command_writer.get_schema()
    }

    pub fn get_index(&self) -> i32 {
        self.command_writer.get_index()
    }

    pub fn get_origin(&self) -> i32 {
        self.command_writer.get_origin()
    }

    pub fn get_identity(&self) -> String {
        self.command_writer.get_identity()
    }

    pub fn get_seq_num(&self) -> i32 {
        self.command_writer.get_seq_num()
    }

    pub async fn run<'b>(
        &mut self,
        parameters: &mut Record<'b>,
        timeout: Duration,
        wait_done: bool,
    ) -> AckCmdResult {
        let identity = self.command_writer.get_identity();
        let origin = self.command_writer.get_origin();
        self.ack_reader.flush();
        match self.command_writer.write(parameters).await {
            Ok(seq_num) => loop {
                if let Some(Value::Record(ack_cmd)) = self.ack_reader.pop_back(false, timeout).await
                {
                    let data_dict: HashMap<String, Value> = ack_cmd
                        .into_iter()
                        .map(|(field, value)| (field, value))
                        .collect();

                    if *data_dict.get("origin").unwrap_or(&Value::Int(0)) == Value::Int(origin)
                        && *data_dict
                            .get("identity")
                            .unwrap_or(&Value::String("".to_owned()))
                            == Value::String(identity.to_owned())
                        && *data_dict.get("private_seqNum").unwrap_or(&Value::Int(0))
                            == Value::Int(seq_num)
                    {
                        let ack = sal_enums::get_ackcmd_code(data_dict.get("ack")).clone();
                        let error: isize = match data_dict.get("error") {
                            Some(Value::Long(error)) => *error as isize,
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

    pub async fn run_typed<'b, T>(
        &mut self,
        data: &mut T,
        timeout: Duration,
        wait_done: bool,
    ) -> AckCmdResult
    where
        T: BaseSALTopic + Serialize + Debug,
    {
        let identity = self.command_writer.get_identity();
        let origin = self.command_writer.get_origin();
        self.ack_reader.flush();
        log::debug!("Sending command...");
        match self.command_writer.write_typed(data).await {
            Ok(seq_num) => {
                log::debug!("Wainting ack for seq_num {seq_num}.");

                loop {
                    if let Some(Value::Record(ack_cmd)) =
                        self.ack_reader.pop_back(false, timeout).await
                    {
                        log::debug!("Got {ack_cmd:?}");

                        let data_dict: HashMap<String, Value> = ack_cmd
                            .into_iter()
                            .map(|(field, value)| (field, value))
                            .collect();

                        if *data_dict.get("origin").unwrap_or(&Value::Int(0)) == Value::Int(origin)
                            && *data_dict
                                .get("identity")
                                .unwrap_or(&Value::String("".to_owned()))
                                == Value::String(identity.to_owned())
                            && *data_dict.get("private_seqNum").unwrap_or(&Value::Int(0))
                                == Value::Int(seq_num)
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

                            log::debug!(
                                "Command ack: {:?}; wait done {} is final? {}, is good? {}",
                                command_ack.get_ack_enum(),
                                wait_done,
                                command_ack.is_final(),
                                command_ack.is_good()
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
                        } else {
                            log::debug!("Discarding ack: {data_dict:?}");
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
                }
            }
            Err(error) => Err(CommandAck::invalid_command(&error.to_string())),
        }
    }
}
