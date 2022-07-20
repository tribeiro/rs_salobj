use crate::topics::field_info;
use crate::topics::sal_objects::{Item, SalTopic};
use std::collections::HashMap;

/// Information about one topic.
pub struct TopicInfo {
    component_name: String,
    topic_subname: String,
    sal_name: String,
    fields: HashMap<String, field_info::FieldInfo>,
    description: String,
    partitions: usize,
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
}
