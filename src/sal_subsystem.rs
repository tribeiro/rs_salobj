use crate::topics::sal_objects::SALObjects;
use crate::topics::topic_info::{self, TopicInfo};
use crate::utils::xml_utils::read_xml_interface;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct SALSubsystemSet {
    #[serde(rename = "$value")]
    sal_subsystems: Vec<SALSubsystem>,
}

#[derive(Deserialize, Debug, Clone)]
struct SALSubsystem {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "AddedGenerics")]
    added_generics: String,
    #[serde(rename = "IndexEnumeration")]
    index_enumeration: String,
}

pub struct SALSubsystemInfo {
    name: String,
    description: String,
    indexed: bool,
    added_generics: String,
    sal_objects_generics: SALObjects,
    sal_objects_component: SALObjects,
}

impl SALSubsystemSet {
    /// Create new SAL Subsystem Set by reading information from
    /// SalSubsystems.xml file.
    pub fn new() -> SALSubsystemSet {
        let xml_interface = read_xml_interface("SALSubsystems.xml").unwrap();
        serde_xml_rs::from_str(&xml_interface).unwrap()
    }

    pub fn get_sal_subsystem_info(self, name: &str) -> SALSubsystemInfo {
        let sal_subsystem_info: Vec<SALSubsystem> = self
            .sal_subsystems
            .into_iter()
            .filter(|s| s.name == name)
            .collect();
        assert_eq!(sal_subsystem_info.len(), 1);

        SALSubsystemInfo {
            name: String::from(&sal_subsystem_info[0].name),
            description: String::from(&sal_subsystem_info[0].description),
            indexed: &sal_subsystem_info[0].index_enumeration != "no",
            added_generics: String::from(&sal_subsystem_info[0].added_generics),
            sal_objects_generics: SALObjects::sal_generics(),
            sal_objects_component: SALObjects::new(&sal_subsystem_info[0].name),
        }
    }
}

impl SALSubsystemInfo {
    /// Return the description field of the component.
    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    /// Is the component indexed?
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    /// Get all commands from the component, including generics.
    pub fn get_commands(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        self.get_commands_generics(topic_subname)
            .into_iter()
            .chain(self.get_commands_component(topic_subname))
            .collect()
    }
    /// Get all events from the component, including generics.
    pub fn get_events(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        self.get_events_generics(topic_subname)
            .into_iter()
            .chain(self.get_events_component(topic_subname))
            .collect()
    }
    /// Get all telemetry from the component, including generics.
    pub fn get_telemetry(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        self.get_telemetry_component(topic_subname)
    }

    /// Get the generic commands from the component.
    ///
    /// This method returns a hashmap with the command name as the key and the
    /// `TopicInfo` as value, which can later be used to construct the topic
    /// type.
    ///
    /// The generic commands are defined from the `AddedGenerics` attribute
    /// in the component definition anf the generics topic set.
    fn get_commands_generics(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        let added_generics = &self.added_generics;

        let generic_commands = self
            .sal_objects_generics
            .get_category_commands(&format!("mandatory, {added_generics}"));

        generic_commands
            .into_iter()
            .filter_map(|(name, sal_topic)| {
                Some((
                    name,
                    TopicInfo::from_generic_sal_topic(
                        &sal_topic,
                        topic_subname,
                        self.indexed,
                        &self.name,
                    ),
                ))
            })
            .collect()
    }

    /// Get the component telemetry topics
    fn get_commands_component(
        &self,
        topic_subname: &str,
    ) -> HashMap<String, topic_info::TopicInfo> {
        let commands = self.sal_objects_component.get_commands();

        commands
            .into_iter()
            .filter_map(|(name, sal_topic)| {
                Some((
                    name,
                    TopicInfo::from_sal_topic(&sal_topic, topic_subname, self.indexed),
                ))
            })
            .collect()
    }

    /// Get the generic events from the component.
    fn get_events_generics(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        let added_generics = &self.added_generics;

        let generic_events = self
            .sal_objects_generics
            .get_category_events(&format!("mandatory, {added_generics}"));

        generic_events
            .into_iter()
            .filter_map(|(name, sal_topic)| {
                Some((
                    name,
                    TopicInfo::from_generic_sal_topic(
                        &sal_topic,
                        topic_subname,
                        self.indexed,
                        &self.name,
                    ),
                ))
            })
            .collect()
    }

    /// Get the component events topics
    fn get_events_component(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        let events = self.sal_objects_component.get_events();

        events
            .into_iter()
            .filter_map(|(name, sal_topic)| {
                Some((
                    name,
                    TopicInfo::from_sal_topic(&sal_topic, topic_subname, self.indexed),
                ))
            })
            .collect()
    }

    /// Get the component telemetry topics
    fn get_telemetry_component(
        &self,
        topic_subname: &str,
    ) -> HashMap<String, topic_info::TopicInfo> {
        let telemetry = self.sal_objects_component.get_telemetry();

        telemetry
            .into_iter()
            .filter_map(|(name, sal_topic)| {
                Some((
                    name,
                    TopicInfo::from_sal_topic(&sal_topic, topic_subname, self.indexed),
                ))
            })
            .collect()
    }

    fn get_generics(&self) -> HashMap<String, topic_info::TopicInfo> {
        let sal_objects_generics = SALObjects::sal_generics();
        HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn get_description() {
        let sal_subsystem_set = SALSubsystemSet::new();
        let sal_subsystem_info = sal_subsystem_set.get_sal_subsystem_info("Test");

        let description = sal_subsystem_info.get_description();

        assert_eq!(
            description,
            "A SAL component designed to support testing SAL itself."
        )
    }

    #[test]
    fn is_indexed() {
        let sal_subsystem_set = SALSubsystemSet::new();
        let sal_subsystem_info = sal_subsystem_set.get_sal_subsystem_info("Test");

        let is_indexed = sal_subsystem_info.is_indexed();

        assert!(is_indexed)
    }

    #[test]
    fn get_commands_generics_test_csc() {
        let sal_subsystem_set = SALSubsystemSet::new();
        let sal_subsystem_info = sal_subsystem_set.get_sal_subsystem_info("Test");

        let commands_generics = sal_subsystem_info.get_commands_generics("unit_test");

        let expected_generic_commands = HashSet::from([
            String::from("SALGeneric_command_disable"),
            String::from("SALGeneric_command_enable"),
            String::from("SALGeneric_command_exitControl"),
            String::from("SALGeneric_command_setAuthList"),
            String::from("SALGeneric_command_setLogLevel"),
            String::from("SALGeneric_command_standby"),
            String::from("SALGeneric_command_start"),
        ]);

        assert_eq!(
            expected_generic_commands,
            commands_generics.keys().cloned().collect(),
        );
    }
    #[test]
    fn get_commands_generics_script_csc() {
        let sal_subsystem_set = SALSubsystemSet::new();
        let sal_subsystem_info = sal_subsystem_set.get_sal_subsystem_info("Script");

        let commands_generics = sal_subsystem_info.get_commands_generics("unit_test");

        let expected_generic_commands = HashSet::from([
            String::from("SALGeneric_command_setAuthList"),
            String::from("SALGeneric_command_setLogLevel"),
        ]);

        assert_eq!(
            expected_generic_commands,
            commands_generics.keys().cloned().collect()
        );
    }
}
