use crate::sal_subsystem;
use crate::topics::topic_info;
use std::collections::HashMap;
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
    fn new(name: &str, topic_subname: &str) -> ComponentInfo {
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
}

#[cfg(test)]

mod tests {

    use super::*;

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
}
