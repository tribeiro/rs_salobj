//! Utilities to read schema files for a specific component.

use crate::error::errors::SalObjError;

use serde_json;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{env, fs};

pub fn glob_schema_files(name: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let schema_dir_path = env::var("LSST_SCHEMA_PATH")?;
    let schema_dir = Path::new(&schema_dir_path).join(name);

    if !schema_dir.exists() {
        return Err(Box::new(SalObjError::new(&format!(
            "Schema path does not exists: {schema_dir_path}."
        ))));
    }

    let json_files: Vec<String> = schema_dir
        .read_dir()?
        .filter_map(|file_name| match file_name {
            Ok(file_name) => {
                let file_path = file_name.path().to_str()?.to_owned();

                if file_path.ends_with(".json") {
                    Some(file_path)
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .collect();

    Ok(json_files
        .into_iter()
        .filter_map(|filename| match fs::read_to_string(&filename) {
            Ok(schema) => {
                let filename = Path::new(&filename)
                    .file_name()?
                    .to_str()?
                    .to_owned()
                    .replace(".json", "");
                Some((filename, schema))
            }
            Err(_) => None,
        })
        .collect())
}

pub fn parse_hash_table(hash_table: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let hash_table_json = serde_json::from_str(hash_table)?;

    if let serde_json::Value::Object(map) = hash_table_json {
        let hash_map: HashMap<String, String> = map
            .into_iter()
            .filter_map(|(key, value)| match value {
                serde_json::Value::String(hash) => Some((key, hash)),
                _ => None,
            })
            .collect();
        Ok(hash_map)
    } else {
        Err(Box::new(SalObjError::new(&format!(
            "Could not parse hash table: {hash_table}"
        ))))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use apache_avro::Schema;
    use std::collections::HashSet;

    macro_rules! set_test_lsst_schema_path {
        () => {
            env::set_var(
                "LSST_SCHEMA_PATH",
                concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/"),
            );
        };
    }

    #[test]
    fn test_parse_hash_map() {
        set_test_lsst_schema_path!();
        let topic_schemas = glob_schema_files("Test").unwrap();
        let hash_map_table_str = topic_schemas.get("Test_hash_table").unwrap();

        let hash_table = parse_hash_table(&hash_map_table_str).unwrap();

        //for (key, value) in hash_table {
        //    println!("{key}: {value}");
        //}
        assert_eq!(hash_table.get("logevent_heartbeat").unwrap(), "9690f77a")
    }

    #[test]
    fn test_glob_schema_files() {
        set_test_lsst_schema_path!();
        let topic_schemas = glob_schema_files("Test").unwrap();

        let expected_topics = HashSet::from([
            String::from("Test_ackcmd"),
            String::from("Test_arrays"),
            String::from("Test_command_disable"),
            String::from("Test_command_enable"),
            String::from("Test_command_exitControl"),
            String::from("Test_command_fault"),
            String::from("Test_command_setArrays"),
            String::from("Test_command_setAuthList"),
            String::from("Test_command_setLogLevel"),
            String::from("Test_command_setScalars"),
            String::from("Test_command_standby"),
            String::from("Test_command_start"),
            String::from("Test_command_wait"),
            String::from("Test_field_enums"),
            String::from("Test_global_enums"),
            String::from("Test_hash_table"),
            String::from("Test_logevent_arrays"),
            String::from("Test_logevent_authList"),
            String::from("Test_logevent_configurationApplied"),
            String::from("Test_logevent_configurationsAvailable"),
            String::from("Test_logevent_errorCode"),
            String::from("Test_logevent_heartbeat"),
            String::from("Test_logevent_logLevel"),
            String::from("Test_logevent_logMessage"),
            String::from("Test_logevent_scalars"),
            String::from("Test_logevent_simulationMode"),
            String::from("Test_logevent_softwareVersions"),
            String::from("Test_logevent_summaryState"),
            String::from("Test_scalars"),
        ]);

        for (topic_name, topic_schema) in &topic_schemas {
            assert!(expected_topics.contains(topic_name));
            if !topic_name.contains("field_enums")
                && !topic_name.contains("global_enums")
                && !topic_name.contains("hash_table")
            {
                let _ = Schema::parse_str(&topic_schema).unwrap();
            }
        }
    }
}
