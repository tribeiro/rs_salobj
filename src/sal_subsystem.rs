use crate::error::errors::{SalObjError, SalObjResult};
use crate::topics::topic_info::{self, TopicInfo};
use crate::utils::schema_utils::{glob_schema_files, parse_hash_table};
use crate::utils::types::SALSubsystemInfoRet;
use crate::utils::xml_utils::convert_sal_name_to_topic_name;
use apache_avro::Schema;
use std::collections::HashMap;

pub struct SALSubsystemInfo {
    name: String,
    indexed: bool,
    topic_schemas: HashMap<String, String>,
    hash_table: HashMap<String, String>,
}

impl SALSubsystemInfo {
    pub fn new(name: &str) -> SALSubsystemInfoRet {
        let topic_schema = glob_schema_files(name)?;
        let hash_table = parse_hash_table(topic_schema.get(&format!("{name}_hash_table")).ok_or(
            SalObjError::new("Could not find hash table for component in schema directory."),
        )?)?;

        let indexed = topic_schema.get(&format!("{name}_logevent_heartbeat")).ok_or(SalObjError::new("No heartbeat topic defined for component. Cannot determine if it is indexed. This is a mandatory topic so something might be wrong with the component topic list."))?.contains("salIndex");

        Ok(SALSubsystemInfo {
            name: name.to_owned(),
            indexed,
            topic_schemas: topic_schema,
            hash_table,
        })
    }

    /// Is the component indexed?
    pub fn is_indexed(&self) -> bool {
        self.indexed
    }

    pub fn get_topic_schemas(&self) -> HashMap<String, String> {
        self.topic_schemas
            .iter()
            .filter_map(|(topic_name, schema_str)| {
                if !topic_name.contains("field_enums")
                    && !topic_name.contains("global_enums")
                    && !topic_name.contains("hash_table")
                {
                    Some((
                        convert_sal_name_to_topic_name(&self.name, topic_name),
                        schema_str.to_owned(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all commands from the component, including generics.
    pub fn get_commands(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        let commands = self
            .topic_schemas
            .iter()
            .filter_map(|(topic_name, topic_schema)| {
                if topic_name.contains("_command_") {
                    if let Ok(schema) = Schema::parse_str(topic_schema) {
                        Some((topic_name.to_owned(), schema))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        self.make_topic_info(commands, topic_subname)
    }

    pub fn get_ackcmd(&self, topic_subname: &str) -> SalObjResult<topic_info::TopicInfo> {
        let ackcmd = Schema::parse_str(self.topic_schemas.get(&format!("{}_ackcmd", self.name)).ok_or(SalObjError::new(&format!("No ackcmd topic found for {}. This is a mandatory topic so something might be wrong with the component topic list.", self.name)))?)?;

        Ok(TopicInfo::new()
            .with_component(&self.name)
            .with_topic_name("ackcmd")
            .with_topic_subname(topic_subname)
            .with_schema(ackcmd)
            .with_rev_code(self.hash_table.get("ackcmd").map(|x| x.as_str()))
            .with_indexed(self.indexed))
    }

    /// Get all events from the component, including generics.
    pub fn get_events(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        let events = self
            .topic_schemas
            .iter()
            .filter_map(|(topic_name, topic_schema)| {
                if topic_name.contains("_logevent_") {
                    if let Ok(schema) = Schema::parse_str(topic_schema) {
                        Some((topic_name.to_owned(), schema))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        self.make_topic_info(events, topic_subname)
    }

    /// Get all telemetry from the component, including generics.
    pub fn get_telemetry(&self, topic_subname: &str) -> HashMap<String, topic_info::TopicInfo> {
        let telemetry = self
            .topic_schemas
            .iter()
            .filter_map(|(topic_name, topic_schema)| {
                if !topic_name.contains("field_enums")
                    && !topic_name.contains("global_enums")
                    && !topic_name.contains("hash_table")
                    && !topic_name.contains("_logevent_")
                    && !topic_name.contains("_command_")
                    && !topic_name.contains("_ackcmd")
                {
                    if let Ok(schema) = Schema::parse_str(topic_schema) {
                        Some((topic_name.to_owned(), schema))
                    } else {
                        None
                    }
                } else {
                    None
                }
            });

        self.make_topic_info(telemetry, topic_subname)
    }

    fn make_topic_info<T>(
        &self,
        topic_schemas: T,
        topic_subname: &str,
    ) -> HashMap<String, topic_info::TopicInfo>
    where
        T: Iterator<Item = (String, Schema)>,
    {
        topic_schemas
            .map(|(name, schema)| {
                (
                    name.to_owned(),
                    TopicInfo::new()
                        .with_component(&self.name)
                        .with_topic_name(&name)
                        .with_topic_subname(topic_subname)
                        .with_schema(schema.to_owned())
                        .with_rev_code(self.hash_table.get(&name).map(|x| x.as_str()))
                        .with_indexed(self.indexed),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn is_indexed() {
        let sal_subsystem_info = SALSubsystemInfo::new("Test").unwrap();

        let is_indexed = sal_subsystem_info.is_indexed();

        assert!(is_indexed)
    }

    #[test]
    fn test_get_commands() {
        let sal_subsystem_info = SALSubsystemInfo::new("Test").unwrap();

        let expected_generic_commands = HashSet::from([
            String::from("Test_command_disable"),
            String::from("Test_command_enable"),
            String::from("Test_command_exitControl"),
            String::from("Test_command_setLogLevel"),
            String::from("Test_command_standby"),
            String::from("Test_command_setScalars"),
            String::from("Test_command_wait"),
            String::from("Test_command_setArrays"),
            String::from("Test_command_fault"),
            String::from("Test_command_start"),
        ]);

        let commands = sal_subsystem_info.get_commands("unit_test");

        assert_eq!(
            expected_generic_commands,
            commands.keys().cloned().collect(),
        );
    }
}
