use crate::{
    sal_info,
    utils::xml_utils::{read_xml_interface, unwrap_xml_interface},
};
use std::{collections::HashMap, hash::Hash};

use super::field_info;

#[derive(Deserialize, Debug)]
pub struct SALObjects {
    #[serde(rename = "SALCommandSet", default = "SALTopicSet::new")]
    sal_command_set: SALTopicSet,
    #[serde(rename = "SALEventSet", default = "SALTopicSet::new")]
    sal_event_set: SALTopicSet,
    #[serde(rename = "SALTelemetrySet", default = "SALTopicSet::new")]
    sal_telemetry_set: SALTopicSet,
}

#[derive(Deserialize, Debug)]
struct SALTopicSet {
    #[serde(rename = "$value")]
    topic_set: Vec<SalTopic>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SalTopic {
    #[serde(rename = "Subsystem")]
    subsystem: String,
    #[serde(rename = "EFDB_Topic")]
    efdb_topic: String,
    #[serde(rename = "Description", default = "String::new")]
    description: String,
    #[serde(rename = "Category", default = "default_category")]
    category: String,
    #[serde(alias = "item", default = "Vec::new")]
    item: Vec<Item>,
}

fn default_category() -> String {
    String::from("NO_CATEGORY")
}

#[derive(Deserialize, Debug, Clone)]
pub struct Item {
    #[serde(rename = "EFDB_Name", default = "String::new")]
    efdb_name: String,
    #[serde(rename = "Description", default = "String::new")]
    description: String,
    #[serde(rename = "IDL_Type", default = "String::new")]
    idl_type: String,
    #[serde(rename = "IDL_Size", default = "default_int")]
    idl_size: usize,
    #[serde(rename = "Units", default = "String::new")]
    units: String,
    #[serde(rename = "Count", default = "default_int")]
    count: usize,
}

fn default_int() -> usize {
    0
}

impl SalTopic {
    /// Return topic name.
    pub fn get_topic_name(&self) -> String {
        let efdb_name = &self.efdb_topic;
        match efdb_name.split_once("_") {
            Some((_, name)) => String::from(name),
            None => panic!(
                "{}",
                format!("Topic name not in expected format <Category>_<type>_<name>: {efdb_name}")
            ),
        }
    }

    /// Get component name/subsystem.
    pub fn get_subsystem(&self) -> String {
        String::from(&self.subsystem)
    }

    /// Return copy of the topic description field.
    pub fn get_description(&self) -> String {
        String::from(&self.description)
    }

    /// Get field info for the associated items.
    pub fn get_field_info(&self) -> HashMap<String, field_info::FieldInfo> {
        self.item
            .clone()
            .into_iter()
            .map(|item| {
                (
                    item.efdb_name.clone(),
                    field_info::FieldInfo::new(
                        &item.efdb_name,
                        &item.idl_type,
                        item.count,
                        &item.units,
                        &item.description,
                    ),
                )
            })
            .collect()
    }

    pub fn get_items(&self) -> Vec<Item> {
        self.item.clone()
    }
}

impl SALObjects {
    /// Get the topic names.
    pub fn get_topic_names(&self) -> HashMap<String, Vec<String>> {
        let sal_command_set = Vec::clone(&self.sal_command_set.topic_set);
        let sal_event_set = Vec::clone(&self.sal_event_set.topic_set);
        let sal_telemetry_set = Vec::clone(&self.sal_telemetry_set.topic_set);

        HashMap::from([
            (
                String::from("commands"),
                sal_command_set.into_iter().map(|s| s.efdb_topic).collect(),
            ),
            (
                String::from("events"),
                sal_event_set.into_iter().map(|s| s.efdb_topic).collect(),
            ),
            (
                String::from("telemetry"),
                sal_telemetry_set
                    .into_iter()
                    .map(|s| s.efdb_topic)
                    .collect(),
            ),
        ])
    }

    /// Get command definitions.
    pub fn get_commands(&self) -> HashMap<String, Vec<Item>> {
        self.sal_command_set.get_items()
    }

    /// Get command definitions.
    pub fn get_category_commands(&self, category: &str) -> HashMap<String, Vec<Item>> {
        self.sal_command_set.get_category_items(category)
    }

    /// Get events definitions.
    pub fn get_events(&self) -> HashMap<String, Vec<Item>> {
        self.sal_event_set.get_items()
    }

    /// Get events definitions.
    pub fn get_category_events(&self, category: &str) -> HashMap<String, Vec<Item>> {
        self.sal_event_set.get_category_items(category)
    }

    /// Get telemetry definitions.
    pub fn get_telemetry(&self) -> HashMap<String, Vec<Item>> {
        self.sal_telemetry_set.get_items()
    }

    /// Get telemetry definitions.
    pub fn get_category_telemetry(&self, category: &str) -> HashMap<String, Vec<Item>> {
        self.sal_telemetry_set.get_category_items(category)
    }

    /// Create SALObjects from SALGenerics.
    pub fn sal_generics() -> SALObjects {
        let xml_interface = read_xml_interface("SALGenerics.xml").unwrap();
        serde_xml_rs::from_str(&xml_interface).unwrap()
    }

    /// Create SALObjects from component name.
    pub fn new(name: &str) -> SALObjects {
        let header = r#"<?xml version="1.0" encoding="UTF-8"?>
        <?xml-stylesheet type="text/xsl" href="http://project.lsst.org/ts/sal_objects/schema/SALCommandSet.xsl"?>
        <SALObjects>
        "#;
        let footer = r#"</SALObjects>
        "#;
        let command_name = format!("{}/{}_Commands.xml", name, name);
        let events_name = format!("{}/{}_Events.xml", name, name);
        let telemetry_name = format!("{}/{}_Telemetry.xml", name, name);
        let command_xml_interface: String = unwrap_xml_interface(read_xml_interface(&command_name));
        let events_xml_interface: String = unwrap_xml_interface(read_xml_interface(&events_name));
        let telemetry_xml_interface: String =
            unwrap_xml_interface(read_xml_interface(&telemetry_name));
        let xml_interface = format!(
            "{}{}{}{}{}",
            header, command_xml_interface, events_xml_interface, telemetry_xml_interface, footer,
        );
        serde_xml_rs::from_str(&xml_interface).unwrap()
    }
}

impl SALTopicSet {
    fn new() -> SALTopicSet {
        SALTopicSet {
            topic_set: Vec::new(),
        }
    }

    /// Get items from topic set.
    fn get_items(&self) -> HashMap<String, Vec<Item>> {
        let items: Vec<(String, Vec<Item>)> = Vec::clone(&self.topic_set)
            .iter()
            .map(|s| (s.efdb_topic.clone(), s.item.clone()))
            .collect();

        HashMap::from_iter(items)
    }

    /// Get items from topic set.
    fn get_category_items(&self, category: &str) -> HashMap<String, Vec<Item>> {
        let items: Vec<(String, Vec<Item>)> = Vec::clone(&self.topic_set)
            .iter()
            .filter(|sal_topic| {
                category.contains(&sal_topic.category)
                    | category.contains(&sal_topic.get_topic_name())
            })
            .map(|s| (s.efdb_topic.clone(), s.item.clone()))
            .collect();
        //.filter(|line| !line.contains("?xml"))
        HashMap::from_iter(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn get_topic_names_sal_generics() {
        let sal_objects = SALObjects::sal_generics();

        let generics = sal_objects.get_topic_names();
        let expected_generic_commands = vec![
            "SALGeneric_command_abort",
            "SALGeneric_command_enable",
            "SALGeneric_command_disable",
            "SALGeneric_command_standby",
            "SALGeneric_command_exitControl",
            "SALGeneric_command_start",
            "SALGeneric_command_enterControl",
            "SALGeneric_command_setLogLevel",
        ];

        let expected_generic_events = vec![
            "SALGeneric_logevent_configurationApplied",
            "SALGeneric_logevent_configurationsAvailable",
            "SALGeneric_logevent_errorCode",
            "SALGeneric_logevent_summaryState",
            "SALGeneric_logevent_logLevel",
            "SALGeneric_logevent_logMessage",
            "SALGeneric_logevent_simulationMode",
            "SALGeneric_logevent_softwareVersions",
            "SALGeneric_logevent_heartbeat",
            "SALGeneric_logevent_authList",
            "SALGeneric_logevent_largeFileObjectAvailable",
            "SALGeneric_logevent_statusCode",
        ];

        for command_name in expected_generic_commands.into_iter() {
            assert!(generics["commands"].contains(&String::from(command_name)));
        }
        for event_name in expected_generic_events.into_iter() {
            assert!(generics["events"].contains(&String::from(event_name)));
        }
    }

    #[test]
    fn get_topic_names_test_component() {
        let sal_objects = SALObjects::new("Test");
        let test_component_topic_names = sal_objects.get_topic_names();

        let expected_test_commands = vec![
            "Test_command_setScalars",
            "Test_command_setArrays",
            "Test_command_fault",
            "Test_command_wait",
        ];

        let expected_test_events = vec!["Test_logevent_scalars", "Test_logevent_arrays"];

        let expected_test_telemetry = vec!["Test_scalars", "Test_arrays"];

        for command_name in expected_test_commands.into_iter() {
            assert!(test_component_topic_names["commands"].contains(&String::from(command_name)));
        }
        for event_name in expected_test_events.into_iter() {
            assert!(test_component_topic_names["events"].contains(&String::from(event_name)));
        }
        for telemetry_name in expected_test_telemetry.into_iter() {
            assert!(test_component_topic_names["telemetry"].contains(&String::from(telemetry_name)));
        }
    }

    #[test]
    fn get_commands_test_component() {
        let sal_objects = SALObjects::new("Test");

        let test_commands = sal_objects.get_commands();

        let test_command_wait = test_commands.get("Test_command_wait").unwrap();

        let expected_items = vec!["ack", "duration"];

        let items: Vec<String> = test_command_wait
            .iter()
            .map(|item| item.efdb_name.clone())
            .collect();

        for item in expected_items.into_iter() {
            assert!(items.contains(&String::from(item)));
        }
    }

    #[test]
    fn get_commands_generics() {
        let sal_objects = SALObjects::sal_generics();
        let generic_commands = sal_objects.get_commands();

        let generic_command_start = generic_commands.get("SALGeneric_command_start").unwrap();

        let expected_items = vec!["configurationOverride"];

        let items: Vec<String> = generic_command_start
            .iter()
            .map(|item| item.efdb_name.clone())
            .collect();

        for item in expected_items.into_iter() {
            assert!(items.contains(&String::from(item)));
        }
    }

    #[test]
    fn get_commands_category_csc() {
        let sal_objects = SALObjects::sal_generics();
        let generic_csc_commands = sal_objects.get_category_commands("csc");

        let expected_csc_generic_commands = HashSet::from([
            String::from("SALGeneric_command_setAuthList"),
            String::from("SALGeneric_command_enable"),
            String::from("SALGeneric_command_disable"),
            String::from("SALGeneric_command_standby"),
            String::from("SALGeneric_command_exitControl"),
            String::from("SALGeneric_command_start"),
            String::from("SALGeneric_command_setLogLevel"),
        ]);

        assert_eq!(
            expected_csc_generic_commands,
            generic_csc_commands.keys().cloned().collect()
        );
    }

    #[test]
    fn get_commands_category_by_name() {
        let sal_objects = SALObjects::sal_generics();
        let category = "command_setAuthList, command_setLogLevel, logevent_authList, logevent_largeFileObjectAvailable";
        let generic_csc_commands = sal_objects.get_category_commands(category);

        let expected_csc_generic_commands = HashSet::from([
            String::from("SALGeneric_command_setAuthList"),
            String::from("SALGeneric_command_setLogLevel"),
        ]);

        assert_eq!(
            expected_csc_generic_commands,
            generic_csc_commands.keys().cloned().collect()
        );
    }

    #[test]
    fn get_events_category_mandatory() {
        let sal_objects = SALObjects::sal_generics();
        let generic_csc_events = sal_objects.get_category_events("mandatory");

        let expected_csc_generics = HashSet::from([
            String::from("SALGeneric_logevent_logLevel"),
            String::from("SALGeneric_logevent_logMessage"),
            String::from("SALGeneric_logevent_heartbeat"),
            String::from("SALGeneric_logevent_softwareVersions"),
        ]);

        assert_eq!(
            expected_csc_generics,
            generic_csc_events.keys().cloned().collect()
        );
    }

    #[test]
    fn get_events_category_csc() {
        let sal_objects = SALObjects::sal_generics();
        let generic_csc_events = sal_objects.get_category_events("csc");

        let expected_csc_generics = HashSet::from([
            String::from("SALGeneric_logevent_errorCode"),
            String::from("SALGeneric_logevent_summaryState"),
            String::from("SALGeneric_logevent_simulationMode"),
            String::from("SALGeneric_logevent_authList"),
        ]);

        assert_eq!(
            expected_csc_generics,
            generic_csc_events.keys().cloned().collect()
        );
    }

    #[test]
    fn get_events_category_by_name() {
        let sal_objects = SALObjects::sal_generics();
        let category = "command_setAuthList, command_setLogLevel, logevent_authList, logevent_largeFileObjectAvailable";
        let generic_csc_events = sal_objects.get_category_events(category);

        let expected_csc_generics = HashSet::from([
            String::from("SALGeneric_logevent_authList"),
            String::from("SALGeneric_logevent_largeFileObjectAvailable"),
        ]);

        assert_eq!(
            expected_csc_generics,
            generic_csc_events.keys().cloned().collect()
        );
    }
}
