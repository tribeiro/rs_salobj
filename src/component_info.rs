//! Information about a SAL Component.
//!
//! A SAL Component is a combination of commands, events and telemetry.
//! This module encapsulates all the information derived from a component
//! schema definition, augmented with the properties that made them a valid
//! SAL Component. For example, [ComponentInfo] includes the acknowledgement
//! topic as well as the component name, topic subname, description and
//! other relevant information.
//!

use crate::{
    error::errors::SalObjResult,
    sal_subsystem,
    topics::topic_info::{self, AvroSchema, TopicInfo},
    utils::xml_utils::convert_sal_name_to_topic_name,
};
use std::collections::HashMap;
extern crate serde;
extern crate serde_xml_rs;

/// Information about one SAL component.
pub struct ComponentInfo {
    name: String,
    topic_subname: String,
    description: String,
    indexed: bool,
    /// Command acknowledgment topic definition.
    ack_cmd: topic_info::TopicInfo,
    /// Map with the commands definition. They key is the
    /// (topic name)[crate::sal_info].
    commands: HashMap<String, topic_info::TopicInfo>,
    /// Map with the events definition. They key is the
    /// (topic name)[crate::sal_info].
    events: HashMap<String, topic_info::TopicInfo>,
    /// Map with the telemetry definition. They key is the
    /// (topic name)[crate::sal_info].
    telemetry: HashMap<String, topic_info::TopicInfo>,
}

impl ComponentInfo {
    pub fn new(name: &str, topic_subname: &str) -> SalObjResult<ComponentInfo> {
        let sal_subsystem_set = sal_subsystem::SALSubsystemSet::new()?;
        let sal_subsystem_info = sal_subsystem_set.get_sal_subsystem_info(name)?;

        let component_commands: HashMap<String, topic_info::TopicInfo> = sal_subsystem_info
            .get_commands(topic_subname)
            .into_iter()
            .map(|(sal_name, items)| (convert_sal_name_to_topic_name(name, &sal_name), items))
            .collect();

        let component_events: HashMap<String, topic_info::TopicInfo> = sal_subsystem_info
            .get_events(topic_subname)
            .into_iter()
            .map(|(sal_name, items)| (convert_sal_name_to_topic_name(name, &sal_name), items))
            .collect();

        let component_telemetry: HashMap<String, topic_info::TopicInfo> = sal_subsystem_info
            .get_telemetry(topic_subname)
            .into_iter()
            .map(|(sal_name, items)| (convert_sal_name_to_topic_name(name, &sal_name), items))
            .collect();

        Ok(ComponentInfo {
            name: String::from(name),
            topic_subname: String::from(topic_subname),
            description: sal_subsystem_info.get_description(),
            indexed: sal_subsystem_info.is_indexed(),
            ack_cmd: topic_info::TopicInfo::get_ackcmd(
                name,
                topic_subname,
                sal_subsystem_info.is_indexed(),
            ),
            commands: component_commands,
            events: component_events,
            telemetry: component_telemetry,
        })
    }

    /// Get ackcmd topic info.
    pub fn get_ackcmd_topic_info(&self) -> &TopicInfo {
        &self.ack_cmd
    }

    /// Get command topic names.
    pub fn get_topic_name_commands(&self) -> Vec<String> {
        self.commands.keys().map(|topic| topic.to_owned()).collect()
    }

    /// Get command topic info.
    pub fn get_topic_info_command(&self, topic_name: &str) -> Option<&TopicInfo> {
        self.commands.get(topic_name)
    }

    /// Get event topic names.
    pub fn get_topic_name_events(&self) -> Vec<String> {
        self.events.keys().map(|topic| topic.to_owned()).collect()
    }

    /// Get event topic info.
    pub fn get_topic_info_event(&self, topic_name: &str) -> Option<&TopicInfo> {
        self.events.get(topic_name)
    }

    /// Get telemetry topic names.
    pub fn get_topic_name_telemetry(&self) -> Vec<String> {
        self.telemetry
            .keys()
            .map(|topic| topic.to_owned())
            .collect()
    }

    /// Get telemetry topic info.
    pub fn get_topic_info_telemetry(&self, topic_name: &str) -> Option<&TopicInfo> {
        self.telemetry.get(topic_name)
    }

    /// Get component name.
    pub fn get_component_name(&self) -> String {
        self.name.to_owned()
    }

    /// Get topic subname.
    pub fn get_topic_subname(&self) -> String {
        self.topic_subname.to_owned()
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// Is the component index?
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Make avro schema for all topics in the component.
    ///
    /// Returns Hashmap with topic name as key and [AvroSchema] as value.
    pub fn make_avro_schema(&self) -> HashMap<String, AvroSchema> {
        HashMap::from([("ackcmd".to_owned(), self.ack_cmd.make_avro_schema())])
            .into_iter()
            .chain(ComponentInfo::make_avro_schema_for_topic_set(
                &self.commands,
            ))
            .chain(ComponentInfo::make_avro_schema_for_topic_set(&self.events))
            .chain(ComponentInfo::make_avro_schema_for_topic_set(
                &self.telemetry,
            ))
            .collect()
    }

    /// Make avro schema for a set of topics (a hashmap of topic_name, topic_info).
    fn make_avro_schema_for_topic_set(
        topic_set: &HashMap<String, TopicInfo>,
    ) -> HashMap<String, AvroSchema> {
        topic_set
            .iter()
            .map(|(topic_name, topic_info)| (topic_name.to_owned(), topic_info.make_avro_schema()))
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use apache_avro::{types::Record, Schema};
    use std::collections::HashSet;

    #[test]
    fn create_test_component_info() {
        let component_info = ComponentInfo::new("Test", "unit_test").unwrap();
        let component_info_commands: Vec<&String> =
            component_info.commands.keys().into_iter().collect();

        assert_eq!(component_info.name, "Test");
        assert_eq!(component_info.topic_subname, "unit_test");
        assert_eq!(component_info.indexed, true);
        assert_eq!(
            component_info.description,
            "A SAL component designed to support testing SAL itself."
        );
        assert_eq!(component_info.ack_cmd.get_topic_name(), "ackcmd");
        assert_eq!(component_info.ack_cmd.get_sal_name(), "Test_ackcmd");
        assert_eq!(component_info.get_topic_subname(), "unit_test");
        assert_eq!(
            component_info.get_description(),
            "A SAL component designed to support testing SAL itself."
        );
        assert!(
            component_info.commands.contains_key("command_start"),
            "{}",
            format!("{component_info_commands:?} has no item 'command_start'")
        );
        assert!(
            component_info.commands.contains_key("command_setScalars"),
            "{}",
            format!("{component_info_commands:?} has no item 'command_setScalars'")
        );
        assert!(component_info.events.contains_key("logevent_heartbeat"));
        assert!(component_info.events.contains_key("logevent_scalars"));
        assert!(component_info.telemetry.contains_key("scalars"));
    }

    #[test]
    fn make_avro_schema() {
        let component_info = ComponentInfo::new("Test", "unit_test").unwrap();

        let avro_schema: HashMap<String, Schema> = component_info
            .make_avro_schema()
            .into_iter()
            .map(|(name, schema)| {
                (
                    name.to_owned(),
                    Schema::parse_str(&serde_json::to_string(&schema).unwrap()).unwrap(),
                )
            })
            .collect();

        let heartbeat_schema = avro_schema.get("logevent_heartbeat").unwrap();
        let heartbeat_record = Record::new(&heartbeat_schema).unwrap();

        let record_fields: HashSet<String> = heartbeat_record
            .fields
            .into_iter()
            .map(|(field, _)| field)
            .collect();
        let expected_fields = HashSet::from([
            String::from("salIndex"),
            String::from("private_sndStamp"),
            String::from("private_rcvStamp"),
            String::from("private_efdStamp"),
            String::from("private_kafkaStamp"),
            String::from("private_seqNum"),
            String::from("private_identity"),
            String::from("private_revCode"),
            String::from("private_origin"),
            String::from("heartbeat"),
        ]);

        assert_eq!(record_fields, expected_fields)
    }
}
