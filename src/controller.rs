//! Basic server-like component.
//!
//! The [Controller] is a server-side tool to implement components in the system.
//! They are basically a mirror of the [crate::remote::Remote], in the sense that they receive commands and outputs events and telemetry.

use std::fmt::Debug;

use crate::{
    domain,
    error::errors::{SalObjError, SalObjResult},
    sal_info,
    topics::{
        base_sal_topic::BaseSALTopic, base_topic::BaseTopic, controller_command::ControllerCommand,
        write_topic::WriteTopic,
    },
    utils::types::{ControllerCommandSet, WriteTopicSet},
};
use apache_avro::{to_value, types::Value};
use serde::Serialize;

pub struct Controller<'a> {
    pub commands: ControllerCommandSet<'a>,
    pub events: WriteTopicSet<'a>,
    pub telemetry: WriteTopicSet<'a>,
}

impl<'a> Controller<'a> {
    pub fn new(
        domain: &mut domain::Domain,
        name: &str,
        index: isize,
    ) -> SalObjResult<Controller<'a>> {
        let sal_info = sal_info::SalInfo::new(name, index)?;

        if let Err(error) = domain.register_topics(&sal_info.get_topics_name()) {
            log::warn!("Failed to register topics: {error:?}. Continuing...");
        }

        let commands: ControllerCommandSet = sal_info
            .get_command_names()
            .into_iter()
            .filter_map(|command_name| {
                if let Ok(controller_command) =
                    ControllerCommand::new(&command_name, domain, &sal_info)
                {
                    Some((command_name.to_owned(), controller_command))
                } else {
                    None
                }
            })
            .collect();

        let events: WriteTopicSet = sal_info
            .get_event_names()
            .into_iter()
            .map(|event_name| {
                (
                    event_name.to_owned(),
                    WriteTopic::new(&event_name, &sal_info, domain),
                )
            })
            .collect();

        let telemetry: WriteTopicSet = sal_info
            .get_telemetry_names()
            .into_iter()
            .map(|telemetry_name| {
                (
                    telemetry_name.to_owned(),
                    WriteTopic::new(&telemetry_name, &sal_info, domain),
                )
            })
            .collect();

        Ok(Controller {
            commands,
            events,
            telemetry,
        })
    }

    pub async fn write_telemetry<T>(&mut self, topic_name: &str, data: T) -> SalObjResult<i32>
    where
        T: BaseSALTopic + Serialize,
    {
        if let Some(writer) = self.telemetry.get_mut(topic_name) {
            if let Ok(data_value) = to_value(data) {
                if let Value::Record(data_record) = data_value {
                    let schema = writer.get_schema().clone();
                    let mut record = WriteTopic::make_data_type(&schema).unwrap();
                    for (field, value) in data_record.into_iter() {
                        record.put(&field, value);
                    }
                    writer.write(&mut record).await
                } else {
                    Err(SalObjError::new("Failed to convert value to record."))
                }
            } else {
                Err(SalObjError::new("Failed to serialize data."))
            }
        } else {
            Err(SalObjError::new(&format!(
                "No telemetry topic {topic_name}"
            )))
        }
    }

    pub async fn write_event<T>(&mut self, topic_name: &str, data: &T) -> SalObjResult<i32>
    where
        T: BaseSALTopic + Serialize + Debug,
    {
        if let Some(writer) = self.events.get_mut(topic_name) {
            writer.write_typed(data).await
        } else {
            Err(SalObjError::new(&format!("No event topic {topic_name}")))
        }
    }

    pub async fn process_command(&mut self, command_name: &str) -> SalObjResult<Value> {
        if let Some(command) = self.commands.get_mut(command_name) {
            command.process_command().await
        } else {
            Err(SalObjError::new(&format!("No command {command_name}.")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create() {
        let mut domain = domain::Domain::new();
        let name = "Test";
        let index = 1;
        let controller = Controller::new(&mut domain, name, index);

        assert!(controller.is_ok())
    }
}
