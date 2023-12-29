//! High-level representation of topics.
//!
//! This module contains utilities to represent a component topic structure
//! in sensible way as well as mechanisms to convert topic definition into
//! avro schema.
//!

use apache_avro::Schema;

use crate::error::errors::{SalObjError, SalObjResult};


/// Information about one topic.
pub struct TopicInfo {
    component_name: String,
    topic_subname: String,
    topic_name: String,
    indexed: bool,
    schema: Option<apache_avro::Schema>,
    rev_code: Option<String>,
    description: String,
    partitions: usize,
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
            indexed: false,
            schema: None,
            rev_code: None,
            description: String::new(),
            partitions: 0,
        }
    }

    pub fn with_component(mut self, component_name: &str) -> Self {
        self.component_name = component_name.to_owned();
        self
    }

    pub fn with_topic_subname(mut self, topic_subname: &str) -> Self {
        self.topic_subname = topic_subname.to_owned();
        self
    }

    pub fn with_topic_name(mut self, topic_name: &str) -> Self {
        self.topic_name = topic_name.to_owned();
        self
    }

    pub fn with_schema(mut self, schema: apache_avro::Schema) -> Self {
        self.schema = Some(schema);
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_owned();
        self
    }

    pub fn with_partitions(mut self, partitions: usize) -> Self {
        self.partitions = partitions;
        self
    }

    pub fn with_indexed(mut self, indexed: bool) -> Self {
        self.indexed = indexed;
        self
    }

    pub fn with_rev_code(mut self, rev_code: Option<&str>) -> Self {
        if let Some(rev_code) = rev_code {
            self.rev_code = Some(rev_code.to_owned());
        }
        self
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

    pub fn get_schema(&self) -> Option<Schema> {
        self.schema.clone()
    }

    /// Get number of partitions in this topic.
    ///
    /// This is a Kafka QoS property.
    pub fn get_partitions(&self) -> usize {
        self.partitions
    }

    /// Get revision code for the topic avro schema.
    pub fn get_rev_code(&self) -> SalObjResult<String> {
        if let Some(rev_code) = &self.rev_code {
            Ok(rev_code.to_owned())
        } else {
            Err(SalObjError::new(&format!(
                "Rev code not set for topic {} for {} component.",
                self.topic_name, self.component_name,
            )))
        }
    }

    /// Make schema for the topic.
    pub fn make_schema(&self) -> SalObjResult<apache_avro::Schema> {
        if let Some(schema) = &self.schema {
            Ok(schema.to_owned())
        } else {
            Err(SalObjError::new(&format!(
                "Schema not set for topic {} for {} component.",
                self.topic_name, self.component_name
            )))
        }
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
            String::from("private_efdStamp"),
            String::from("private_kafkaStamp"),
            String::from("private_seqNum"),
            String::from("private_identity"),
            String::from("private_revCode"),
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

    // #[test]
    // fn get_private_fields_indexed() {
    //     let private_fields = TopicInfo::get_private_fields(true);

    //     assert!(private_fields.contains_key("salIndex"))
    // }

    // #[test]
    // fn get_private_fields_not_indexed() {
    //     let private_fields = TopicInfo::get_private_fields(false);

    //     assert!(!private_fields.contains_key("salIndex"))
    // }

    // fn check_ackcmd_fields_indexed(indexed: bool) -> bool {
    //     let ackcmd_fields = TopicInfo::get_ackcmd_fields(indexed);

    //     let expected_ackcmd_field = get_expected_ackcmd_fields(indexed);

    //     expected_ackcmd_field == ackcmd_fields.keys().cloned().collect()
    // }

    // #[test]
    // fn get_ackcmd_fields_indexed() {
    //     assert!(check_ackcmd_fields_indexed(true))
    // }

    // #[test]
    // fn get_ackcmd_fields_not_indexed() {
    //     assert!(check_ackcmd_fields_indexed(false))
    // }

    // #[test]
    // fn get_ackcmd_indexed() {
    //     let ack_cmd = TopicInfo::get_ackcmd("Test", "unit_test", true);

    //     let expected_ackcmd_field = get_expected_ackcmd_fields(true);

    //     assert_eq!(ack_cmd.component_name, "Test");
    //     assert_eq!(ack_cmd.topic_subname, "unit_test");
    //     assert_eq!(ack_cmd.topic_name, "ackcmd");
    //     assert_eq!(ack_cmd.description, "Command acknowledgement");
    //     assert_eq!(
    //         expected_ackcmd_field,
    //         ack_cmd.fields.keys().cloned().collect()
    //     );
    // }

    // #[test]
    // fn get_ackcmd_not_indexed() {
    //     let ack_cmd = TopicInfo::get_ackcmd("Test", "unit_test", false);

    //     let expected_ackcmd_field = get_expected_ackcmd_fields(false);

    //     assert_eq!(ack_cmd.component_name, "Test");
    //     assert_eq!(ack_cmd.topic_subname, "unit_test");
    //     assert_eq!(ack_cmd.description, "Command acknowledgement");
    //     assert_eq!(
    //         expected_ackcmd_field,
    //         ack_cmd.fields.keys().cloned().collect()
    //     );
    // }

    // #[test]
    // fn make_avro_schema() {
    //     let ack_cmd = TopicInfo::get_ackcmd("Test", "unit_test", false);
    //     let ack_cmd_schema = serde_json::to_string(&ack_cmd.make_avro_schema()).unwrap();
    //     let schema = Schema::parse_str(&ack_cmd_schema).unwrap();
    //     let record = Record::new(&schema).unwrap();

    //     let record_fields: HashSet<String> =
    //         record.fields.into_iter().map(|(field, _)| field).collect();
    //     let expected_fields = ack_cmd.get_fields_name();
    //     assert_eq!(record_fields, expected_fields)
    // }

    // #[test]
    // fn test_get_rev_code() {
    //     let ack_cmd = TopicInfo::get_ackcmd("Test", "unit_test", false);
    //     let ack_cmd_rev_code = ack_cmd.get_rev_code().unwrap();
    //     assert_eq!(ack_cmd_rev_code, "abd3610e")
    // }
}
