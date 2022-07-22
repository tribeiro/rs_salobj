use crate::topics::field_info;
use crate::topics::sal_objects::SalTopic;
use std::collections::{HashMap, HashSet};

/// Information about one topic.
pub struct TopicInfo {
    component_name: String,
    topic_subname: String,
    sal_name: String,
    fields: HashMap<String, field_info::FieldInfo>,
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

impl TopicInfo {
    /// Create a new empty instance of `TopicInfo`.
    pub fn new() -> TopicInfo {
        TopicInfo {
            component_name: String::new(),
            topic_subname: String::new(),
            sal_name: String::new(),
            fields: HashMap::new(),
            description: String::new(),
            partitions: 0,
        }
    }

    pub fn get_ackcmd(component_name: &str, topic_subname: &str, indexed: bool) -> TopicInfo {
        TopicInfo {
            component_name: String::from(component_name),
            topic_subname: String::from(topic_subname),
            sal_name: String::from("ackcmd"),
            fields: TopicInfo::get_ackcmd_fields(indexed),
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
            sal_name: sal_topic.get_topic_name(),
            fields: private_fields.into_iter().chain(fields).collect(),
            description: sal_topic.get_description(),
            partitions: 1,
        }
    }

    pub fn get_sal_name(&self) -> String {
        return self.sal_name.clone();
    }

    pub fn get_fields_name(&self) -> HashSet<String> {
        self.fields.keys().cloned().collect()
    }

    /// Make avro schema fpr the topic.
    pub fn make_avro_schema(&self) -> AvroSchema {
        let topic_subname = &self.topic_subname;
        let component_name = &self.component_name;

        AvroSchema {
            avro_message_type: "record".to_owned(),
            name: self.sal_name.to_owned(),
            namespace: format!("lsst.sal.{topic_subname}.{component_name}"),
            fields: self
                .fields
                .iter()
                .map(|(_, field_info)| field_info.make_avro_schema())
                .collect(),
            description: self.description.to_owned(),
        }
    }

    /// Get private fields.
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
    use avro_rs::{types::Record, Schema};
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
        assert_eq!(ack_cmd.sal_name, "ackcmd");
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
