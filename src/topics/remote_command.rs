use crate::{
    domain::Domain,
    sal_enums,
    sal_info::SalInfo,
    topics::{read_topic::ReadTopic, write_topic::WriteTopic},
    utils::command_ack::CommandAck,
};
use apache_avro::types::{Record, Value};
use std::{collections::HashMap, time::Duration};

type Result<T> = std::result::Result<T, T>;

pub struct RemoteCommand {
    command_writer: WriteTopic,
    ack_reader: ReadTopic,
}

impl RemoteCommand {
    pub fn new(command_name: &str, domain: &Domain, sal_info: &SalInfo) -> RemoteCommand {
        RemoteCommand {
            command_writer: WriteTopic::new(command_name, sal_info, domain),
            ack_reader: ReadTopic::new("ackcmd", sal_info.clone(), domain.clone(), 0),
        }
    }

    pub async fn run<'a, 'b>(
        &mut self,
        parameters: &mut Record<'b>,
        timeout: Duration,
        wait_done: bool,
        sal_info: &SalInfo<'a>,
    ) -> Result<CommandAck> {
        let seq_num = self.command_writer.write(parameters, sal_info).await;

        let identity = self.command_writer.get_identity();
        let origin = self.command_writer.get_origin();

        loop {
            if let Some(Value::Record(ack_cmd)) =
                self.ack_reader.pop_back(false, timeout, sal_info).await
            {
                let data_dict: HashMap<String, Value> = ack_cmd
                    .into_iter()
                    .map(|(field, value)| (field, value))
                    .collect();
                if *data_dict.get("origin").unwrap() == Value::Long(origin)
                    && *data_dict.get("identity").unwrap() == Value::String(identity.to_owned())
                    && *data_dict.get("private_seqNum").unwrap() == Value::Long(seq_num)
                {
                    let ack = sal_enums::get_ackcmd_code(data_dict.get("ack").unwrap()).clone();
                    let error: isize = match data_dict.get("error").unwrap() {
                        Value::Int(error) => *error as isize,
                        _ => 0,
                    };
                    let result = match data_dict.get("result").unwrap() {
                        Value::String(result) => result.to_owned(),
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
        }
    }
}
