use crate::sal_subsystem;
use crate::topics::topic_info::{self, AvroSchema, TopicInfo};
use std::collections::HashMap;
use std::hash::Hash;
extern crate serde;
extern crate serde_xml_rs;

/// Information about one SAL component.
pub struct ComponentInfo {
    name: String,
    topic_subname: String,
    description: String,
    indexed: bool,
    ack_cmd: topic_info::TopicInfo,
    commands: HashMap<String, topic_info::TopicInfo>,
    events: HashMap<String, topic_info::TopicInfo>,
    telemetry: HashMap<String, topic_info::TopicInfo>,
}

impl ComponentInfo {
    pub fn new(name: &str, topic_subname: &str) -> ComponentInfo {
        let sal_subsystem_set = sal_subsystem::SALSubsystemSet::new();
        let sal_subsystem_info = sal_subsystem_set.get_sal_subsystem_info(name);

        let component_commands: HashMap<String, topic_info::TopicInfo> = sal_subsystem_info
            .get_commands(topic_subname)
            .into_iter()
            .filter_map(|(topic_name, items)| {
                Some((topic_name.replace("SALGeneric", &name), items))
            })
            .collect();

        let component_events: HashMap<String, topic_info::TopicInfo> = sal_subsystem_info
            .get_events(topic_subname)
            .into_iter()
            .filter_map(|(topic_name, items)| {
                Some((topic_name.replace("SALGeneric", &name), items))
            })
            .collect();

        let component_telemetry: HashMap<String, topic_info::TopicInfo> = sal_subsystem_info
            .get_telemetry(topic_subname)
            .into_iter()
            .filter_map(|(topic_name, items)| {
                Some((topic_name.replace("SALGeneric", &name), items))
            })
            .collect();

        ComponentInfo {
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
        }
    }

    /// Make avro schema for all topics in the component.
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
    use avro_rs::{types::Record, Schema};
    use std::collections::HashSet;

    #[test]
    fn create_test_component_info() {
        let component_info = ComponentInfo::new("Test", "unit_test");
        let component_info_commands: Vec<&String> =
            component_info.commands.keys().into_iter().collect();

        assert_eq!(component_info.name, "Test");
        assert_eq!(component_info.topic_subname, "unit_test");
        assert_eq!(component_info.indexed, true);
        assert_eq!(
            component_info.description,
            "A SAL component designed to support testing SAL itself."
        );
        assert_eq!(component_info.ack_cmd.get_sal_name(), "ackcmd");
        assert!(
            component_info.commands.contains_key("Test_command_start"),
            "{}",
            format!("{component_info_commands:?} has no item 'Test_command_start'")
        );
        assert!(
            component_info
                .commands
                .contains_key("Test_command_setScalars"),
            "{}",
            format!("{component_info_commands:?} has no item 'Test_command_setScalars'")
        );
        assert!(component_info
            .events
            .contains_key("Test_logevent_heartbeat"));
        assert!(component_info.events.contains_key("Test_logevent_scalars"));
        assert!(component_info.telemetry.contains_key("Test_scalars"));
    }

    #[test]
    fn make_avro_schema() {
        let component_info = ComponentInfo::new("Test", "unit_test");

        let avro_schema = component_info.make_avro_schema();
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

        let heartbeat_schema = avro_schema.get("Test_logevent_heartbeat").unwrap();
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
            String::from("private_seqNum"),
            String::from("private_identity"),
            String::from("private_origin"),
            String::from("heartbeat"),
        ]);

        assert_eq!(record_fields, expected_fields)
    }
}
