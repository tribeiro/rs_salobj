//! High-level representation of topics.
//!
//! This module contains utilities to represent a component topic structure
//! in sensible way as well as mechanisms to convert topic definition into
//! avro schema.
//!

use crate::topics::sal_objects::SalTopic;
use crate::{
    error::errors::{SalObjError, SalObjResult},
    topics::field_info,
};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Information about one topic.
pub struct TopicInfo {
    component_name: String,
    topic_subname: String,
    topic_name: String,
    fields: BTreeMap<String, field_info::FieldInfo>,
    description: String,
    partitions: usize,
}

/// Avro schema for topics
#[derive(Serialize, Deserialize, Debug)]
pub struct AvroSchema {
    #[serde(rename = "type")]
    avro_message_type: String,
    name: String,
    namespace: String,
    fields: Vec<field_info::AvroSchemaField>,
    description: String,
}

impl Default for TopicInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicInfo {
    /// Create a new empty instance of `TopicInfo`.
    pub fn new() -> TopicInfo {
        TopicInfo {
            component_name: String::new(),
            topic_subname: String::new(),
            topic_name: String::new(),
            fields: BTreeMap::new(),
            description: String::new(),
            partitions: 0,
        }
    }

    /// Create [TopicInfo] for ackcmd topic.
    ///
    /// This topic is not defined in any component schema only in code.
    ///
    /// See (SalInfo)[crate::sal_info] for more information.
    pub fn get_ackcmd(component_name: &str, topic_subname: &str, indexed: bool) -> TopicInfo {
        TopicInfo {
            component_name: String::from(component_name),
            topic_subname: String::from(topic_subname),
            topic_name: String::from("ackcmd"),
            fields: TopicInfo::get_ackcmd_fields(indexed).into_iter().collect(),
            description: String::from("Command acknowledgement"),
            partitions: 1,
        }
    }

    /// Create topic info from `SalTopic`.
    pub fn from_sal_topic(sal_topic: &SalTopic, topic_subname: &str, indexed: bool) -> TopicInfo {
        let private_fields = TopicInfo::get_private_fields(indexed);

        let fields: HashMap<String, field_info::FieldInfo> = sal_topic.get_field_info();

        TopicInfo {
            component_name: sal_topic.get_subsystem(),
            topic_subname: String::from(topic_subname),
            topic_name: sal_topic.get_topic_name(),
            fields: private_fields.into_iter().chain(fields).collect(),
            description: sal_topic.get_description(),
            partitions: 1,
        }
    }

    /// Create topic info from `SalTopic`.
    pub fn from_generic_sal_topic(
        sal_topic: &SalTopic,
        topic_subname: &str,
        indexed: bool,
        component_name: &str,
    ) -> TopicInfo {
        let private_fields = TopicInfo::get_private_fields(indexed);

        let fields: HashMap<String, field_info::FieldInfo> = sal_topic.get_field_info();

        TopicInfo {
            component_name: component_name.to_owned(),
            topic_subname: String::from(topic_subname),
            topic_name: sal_topic.get_topic_name(),
            fields: private_fields.into_iter().chain(fields).collect(),
            description: sal_topic.get_description(),
            partitions: 1,
        }
    }

    /// Get topic name.
    ///
    /// See (SalInfo)[crate::sal_info] for more information on topic naming
    /// conventions.
    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }

    /// Get topic subname.
    ///
    /// See (SalInfo)[crate::sal_info] for more information on topic naming
    /// conventions.
    pub fn get_topic_subname(&self) -> &str {
        &self.topic_subname
    }

    /// Get a topic sal name.
    ///
    /// See (SalInfo)[crate::sal_info] for more information on topic naming
    /// conventions.
    pub fn get_sal_name(&self) -> String {
        format!("{}_{}", self.component_name, self.topic_name)
    }

    /// Get a [HashSet] with the names of all fields in this topic.
    pub fn get_fields_name(&self) -> HashSet<String> {
        self.fields.keys().cloned().collect()
    }

    /// Get number of partitions in this topic.
    ///
    /// This is a Kafka QoS property.
    pub fn get_partitions(&self) -> usize {
        self.partitions
    }

    /// Make avro schema for the topic.
    pub fn make_avro_schema(&self) -> AvroSchema {
        let component_name = &self.component_name;

        AvroSchema {
            avro_message_type: "record".to_owned(),
            name: self.topic_name.to_owned(),
            namespace: format!("lsst.sal.kafka-{component_name}"),
            fields: self
                .fields
                .values()
                .map(|field_info| field_info.make_avro_schema())
                .collect(),
            description: self.description.to_owned(),
        }
    }

    /// Make schema for the topic.
    pub fn make_schema(&self) -> SalObjResult<apache_avro::Schema> {
        match serde_json::to_string(&self.make_avro_schema()) {
            Ok(avro_schema) => match apache_avro::Schema::parse_str(&avro_schema) {
                Ok(avro_schema) => Ok(avro_schema),
                Err(error) => Err(SalObjError::from_error(error)),
            },
            Err(error) => Err(SalObjError::from_error(error)),
        }
    }

    /// Get private fields.
    ///
    /// Private fields are additional fields included in SAL topics that are
    /// not defined in the topic basic structure. In a sense having these
    /// private fields is what makes a topic a SAL topic, in addition to
    /// conforming with (naming)[crate::sal_info] conventions and other
    /// details.
    fn get_private_fields(indexed: bool) -> HashMap<String, field_info::FieldInfo> {
        let private_fields = HashMap::from([
            (
                String::from("salIndex"),
                field_info::FieldInfo::new(
                    "salIndex",
                    "long",
                    1,
                    "unitless",
                    "SAL index (only present for indexed SAL components)",
                ),
            ),
            (
                String::from("private_sndStamp"),
                field_info::FieldInfo::new(
                    "private_sndStamp",
                    "double",
                    1,
                    "second",
                    "Time of instance publication",
                ),
            ),
            (
                String::from("private_rcvStamp"),
                field_info::FieldInfo::new(
                    "private_rcvStamp",
                    "double",
                    1,
                    "second",
                    "Time of instance reception",
                ),
            ),
            (
                String::from("private_seqNum"),
                field_info::FieldInfo::new(
                    "private_seqNum",
                    "long",
                    1,
                    "unitless",
                    "Sequence number",
                ),
            ),
            (
                String::from("private_identity"),
                field_info::FieldInfo::new(
                    "private_identity",
                    "string",
                    1,
                    "unitless",
                    "Identity of publisher: SAL component name for a CSC or user@host for a user",
                ),
            ),
            (
                String::from("private_origin"),
                field_info::FieldInfo::new(
                    "private_origin",
                    "long",
                    1,
                    "unitless",
                    "Process ID of publisher",
                ),
            ),
        ]);

        // If not indexed remove the "salIndex" element from the private
        // fields.
        if !indexed {
            private_fields
                .into_iter()
                .filter(|name_field_info| name_field_info.0 != "salIndex")
                .collect()
        } else {
            private_fields
        }
    }

    /// Get the definition of ack cmd topic.
    fn get_ackcmd_fields(indexed: bool) -> HashMap<String, field_info::FieldInfo> {
        let ackcmd_fields = HashMap::from([
            (
                String::from("ack"),
                field_info::FieldInfo::new("ack", "long", 1, "unitless", "Acknowledgement code"),
            ),
            (
                String::from("error"),
                field_info::FieldInfo::new(
                    "error",
                    "long",
                    1,
                    "unitless",
                    "An error code; only relevant if ack=FAILED",
                ),
            ),
            (
                String::from("result"),
                field_info::FieldInfo::new("result", "string", 1, "Message", "unitless"),
            ),
            (
                String::from("identity"),
                field_info::FieldInfo::new(
                    "identity",
                    "string",
                    1,
                    "unitless",
                    "private_identity field of the command being acknowledged",
                ),
            ),
            (
                String::from("origin"),
                field_info::FieldInfo::new(
                    "origin",
                    "long",
                    1,
                    "unitless",
                    "private_origin field of the command being acknowledged",
                ),
            ),
            (
                String::from("cmdtype"),
                field_info::FieldInfo::new(
                    "cmdtype",
                    "long",
                    1,
                    "unitless",
                    "Index of command in alphabetical list of commands, with 0 being the first",
                ),
            ),
            (
                String::from("timeout"),
                field_info::FieldInfo::new(
                    "timeout",
                    "double",
                    1,
                    "second",
                    "Estimated remaining duration of command; only relevant if ack=INPROGRESS",
                ),
            ),
        ]);

        ackcmd_fields
            .into_iter()
            .chain(TopicInfo::get_private_fields(indexed))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use apache_avro::{types::Record, Schema};
    use std::collections::HashSet;

    fn get_expected_ackcmd_fields(indexed: bool) -> HashSet<String> {
        let expected_ackcmd_fields = HashSet::from([
            String::from("ack"),
            String::from("error"),
            String::from("result"),
            String::from("identity"),
            String::from("origin"),
            String::from("cmdtype"),
            String::from("timeout"),
        ]);

        let expected_private_fields = get_expected_private_fields(indexed);

        expected_ackcmd_fields
            .into_iter()
            .chain(expected_private_fields)
            .collect()
    }

    fn get_expected_private_fields(indexed: bool) -> HashSet<String> {
        let expected_private_fields = HashSet::from([
            String::from("salIndex"),
            String::from("private_sndStamp"),
            String::from("private_rcvStamp"),
            String::from("private_seqNum"),
            String::from("private_identity"),
            String::from("private_origin"),
        ]);

        if !indexed {
            expected_private_fields
                .into_iter()
                .filter(|name| name != "salIndex")
                .collect()
        } else {
            expected_private_fields
        }
    }

    #[test]
    fn get_private_fields_indexed() {
        let private_fields = TopicInfo::get_private_fields(true);

        assert!(private_fields.contains_key("salIndex"))
    }

    #[test]
    fn get_private_fields_not_indexed() {
        let private_fields = TopicInfo::get_private_fields(false);

        assert!(!private_fields.contains_key("salIndex"))
    }

    fn check_ackcmd_fields_indexed(indexed: bool) -> bool {
        let ackcmd_fields = TopicInfo::get_ackcmd_fields(indexed);

        let expected_ackcmd_field = get_expected_ackcmd_fields(indexed);

        expected_ackcmd_field == ackcmd_fields.keys().cloned().collect()
    }

    #[test]
    fn get_ackcmd_fields_indexed() {
        assert!(check_ackcmd_fields_indexed(true))
    }

    #[test]
    fn get_ackcmd_fields_not_indexed() {
        assert!(check_ackcmd_fields_indexed(false))
    }

    #[test]
    fn get_ackcmd_indexed() {
        let ack_cmd = TopicInfo::get_ackcmd("Test", "unit_test", true);

        let expected_ackcmd_field = get_expected_ackcmd_fields(true);

        assert_eq!(ack_cmd.component_name, "Test");
        assert_eq!(ack_cmd.topic_subname, "unit_test");
        assert_eq!(ack_cmd.topic_name, "ackcmd");
        assert_eq!(ack_cmd.description, "Command acknowledgement");
        assert_eq!(
            expected_ackcmd_field,
            ack_cmd.fields.keys().cloned().collect()
        );
    }

    #[test]
    fn get_ackcmd_not_indexed() {
        let ack_cmd = TopicInfo::get_ackcmd("Test", "unit_test", false);

        let expected_ackcmd_field = get_expected_ackcmd_fields(false);

        assert_eq!(ack_cmd.component_name, "Test");
        assert_eq!(ack_cmd.topic_subname, "unit_test");
        assert_eq!(ack_cmd.description, "Command acknowledgement");
        assert_eq!(
            expected_ackcmd_field,
            ack_cmd.fields.keys().cloned().collect()
        );
    }

    #[test]
    fn make_avro_schema() {
        let ack_cmd = TopicInfo::get_ackcmd("Test", "unit_test", false);
        let ack_cmd_schema = serde_json::to_string(&ack_cmd.make_avro_schema()).unwrap();
        let schema = Schema::parse_str(&ack_cmd_schema).unwrap();
        let record = Record::new(&schema).unwrap();

        let record_fields: HashSet<String> =
            record.fields.into_iter().map(|(field, _)| field).collect();
        let expected_fields = ack_cmd.get_fields_name();
        assert_eq!(record_fields, expected_fields)
    }
}
