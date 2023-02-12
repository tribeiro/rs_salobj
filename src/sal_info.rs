//! Store information about one SAL component and index.
//!
//! A SAL component is mostly defined by its name, which maps to an interface.
//! A component interface is basically a collection of topics. Topics in SAL
//! comes in 4 different forms; commands, command acknowledgement, events and
//! telemetry. Each topic category maps to a set of Quality of Service, in the
//! message passing system and also defines how data should be handled by
//! components.
//!
//! The SalInfo module keeps track of a component interface and provides
//! utility methods to operate with it.
//!
//! # Topic Naming Convention
//!
//! SAL topics follow a specific naming convention and, internally, are used in
//! different contexts. The following is a census of the different ways topics
//! are referred to in the code.
//!
//! * `topic_name`: This is the name of the topic preceded by the type, when it
//! is an event or a command.
//!
//!   Basically:
//!
//!   * `logevent_scalars`: Event named `scalars`.
//!   * `scalars`: Telemetry named `scalars`.
//!   * `command_setScalars`: Command named `setScalars`.
//!
//! * `sal_name`: This is the `topic_name` preceded by the name of the
//! component.
//!
//!   For example:
//!
//!   * `Test_logevent_scalars`.
//!   * `Test_scalars`.
//!   * `Test_command_setScalars`.
//!
//! * `schema_registry_name`: The name of the topic in the schema registry.
//! This is composed of the static string `lsst`, the topic subname, the
//! component name and the topic name separated by "dots".
//!
//!   For example:
//!
//!   * `lsst.test.Test.logevent_scalars`.
//!   * `lsst.test.Test.scalars`.
//!   * `lsst.test.Test.command_setScalars`.
//!
//!   In the cases above the topic subname is `test`. This is controlled by the
//! environment variable `LSST_TOPIC_SUBNAME` and allows us to "namespace" the
//! topics.
//!
//! * `subject_name`: This is the name used to register the topic in the kafka
//! broker. This is composed of the static string `-value` appended to the
//! `schema_registry_name`, e.g.:
//!
//!   * `lsst.test.Test.logevent_scalars-value`.
//!   * `lsst.test.Test.scalars-value`.
//!   * `lsst.test.Test.command_setScalars-value`.
//!
//! * `namespace`: The namespace of the topic schema. This is used in the topic
//! avro schema. This consists of the component name appended to the static
//! string "lsst.sal.kafka-".
//!
//!   For example:
//!
//!   * `lsst.ts.kafka-Test`.
//!

use crate::sal_enums;
use crate::topics::topic_info::TopicInfo;
use crate::{component_info::ComponentInfo, domain::Domain};
use apache_avro::{
    types::{Record, Value},
    Schema,
};
use schema_registry_converter::{
    async_impl::{
        avro::{AvroDecoder, AvroEncoder},
        schema_registry::{post_schema, SrSettings},
    },
    avro_common::DecodeResult,
    error::SRCError,
    schema_registry_common::{RegisteredSchema, SchemaType, SubjectNameStrategy, SuppliedSchema},
};
use std::collections::HashMap;
use std::env;

///Information for one SAL component and index.
pub struct SalInfo<'a> {
    index: isize,
    component_info: ComponentInfo,
    /// HashMap with topics schema, key is topic name.
    topic_schema: HashMap<String, Schema>,
    encoder: AvroEncoder<'a>,
    decoder: AvroDecoder<'a>,
}

impl<'a> SalInfo<'a> {
    /// Create a new instance of `SalInfo`.
    pub fn new(name: &str, index: isize) -> SalInfo<'a> {
        let topic_subname = match env::var("LSST_TOPIC_SUBNAME") {
            Ok(val) => val,
            Err(_) => panic!("You must define environment variable LSST_TOPIC_SUBNAME"),
        };
        let component_info = ComponentInfo::new(name, &topic_subname);

        if index != 0 && !component_info.is_indexed() {
            panic!("Invalid index={index}. Component {name} is not indexed. Index must be 0.")
        }

        let topic_schema = component_info
            .make_avro_schema()
            .into_iter()
            .map(|(topic, avro_schema)| {
                (
                    topic,
                    Schema::parse_str(&serde_json::to_string(&avro_schema).unwrap()).unwrap(),
                )
            })
            .collect();

        SalInfo {
            index,
            component_info,
            topic_schema,
            encoder: SalInfo::make_encoder(),
            decoder: SalInfo::make_decoder(),
        }
    }

    /// Make an AckCmd `Record` from keyword arguments.
    ///
    /// A `Record` is an object that is built from the avro schema and,
    /// therefore, can be published directly afterwards.
    pub fn make_ackcmd(
        &self,
        private_seqnum: i32,
        ack: sal_enums::SalRetCode,
        error: i32,
        result: &str,
        timeout: f32,
    ) -> Record {
        let mut record = Record::new(self.topic_schema.get("ackcmd").unwrap()).unwrap();
        record.put("private_seqNum", Value::Int(private_seqnum));
        record.put("ack", Value::Int(ack as i32));
        record.put("error", Value::Int(error));
        record.put("result", Value::String(result.to_owned()));
        record.put("timeout", Value::Float(timeout));

        record
    }

    /// Is the component indexed?
    pub fn is_indexed(&self) -> bool {
        self.component_info.is_indexed()
    }

    /// Get the component index.
    pub fn get_index(&self) -> isize {
        self.index
    }

    /// Get the component description
    pub fn get_description(&self) -> &str {
        self.component_info.get_description()
    }

    /// Get name\[:index\]
    ///
    /// The suffix is only passed if the component is index.
    pub fn get_name_index(&self) -> String {
        if self.is_indexed() {
            format!(
                "{}:{}",
                self.component_info.get_component_name(),
                self.index
            )
        } else {
            self.component_info.get_component_name()
        }
    }

    /// Get component name.
    pub fn get_name(&self) -> String {
        self.component_info.get_component_name()
    }

    /// Make schema registry topic name
    pub fn make_schema_registry_topic_name(&self, topic_name: &str) -> String {
        format!(
            "lsst.{}.{}.{}",
            self.component_info.get_topic_subname(),
            self.component_info.get_component_name(),
            topic_name
        )
        // "lsst.test.Test.logevent_heartbeat".to_owned()
    }

    pub fn get_sal_name(&self, topic_name: &str) -> String {
        format!("{}_{}", self.get_name(), topic_name)
    }

    /// Make topic subject name.
    pub fn make_subject_name(&self, topic_name: &str) -> String {
        format!(
            "{}-value",
            self.make_schema_registry_topic_name(topic_name)
                .replace(&format!(".{}_", self.get_name()), ".")
        )
    }

    /// Get name of all commands topics.
    pub fn get_command_names(&self) -> Vec<String> {
        self.component_info.get_topic_name_commands()
    }

    /// Get names of all events topics.
    pub fn get_event_names(&self) -> Vec<String> {
        self.component_info.get_topic_name_events()
    }

    /// Get names of all telemetry topics.
    pub fn get_telemetry_names(&self) -> Vec<String> {
        self.component_info.get_topic_name_telemetry()
    }

    /// Get names of all the topics.
    pub fn get_topics_name(&self) -> Vec<String> {
        self.get_telemetry_names()
            .into_iter()
            .map(|topic_name| self.make_schema_registry_topic_name(&topic_name))
            .chain(
                self.get_event_names()
                    .into_iter()
                    .map(|topic_name| self.make_schema_registry_topic_name(&topic_name)),
            )
            .chain(
                self.get_command_names()
                    .into_iter()
                    .map(|topic_name| self.make_schema_registry_topic_name(&topic_name)),
            )
            .chain(vec![self.make_schema_registry_topic_name("ackcmd")])
            .collect()
    }

    /// Get topic info for a particular topic.
    ///
    /// This high-level method will identify if a topic is a command, event,
    /// telemetry or ackcmd and return the appropriate TopicInfo.
    pub fn get_topic_info(&self, topic_name: &str) -> Option<&TopicInfo> {
        if self.is_ackcmd(topic_name) {
            Some(self.component_info.get_ackcmd_topic_info())
        } else if self.is_command(topic_name) {
            self.get_command_topic_info(topic_name)
        } else if self.is_event(topic_name) {
            self.get_event_topic_info(topic_name)
        } else {
            self.get_telemetry_topic_info(topic_name)
        }
    }

    /// Check if topic name matches command acknowledgement.
    fn is_ackcmd(&self, topic_name: &str) -> bool {
        topic_name == "ackcmd"
    }

    /// Check if topic name matches command name.
    ///
    /// This method does not test if the topic is a valid topic from the
    /// component.
    pub fn is_command(&self, topic_name: &str) -> bool {
        topic_name.starts_with("command_")
    }

    /// Check if topic name matches event name.
    ///
    /// This method does not test if the topic is a valid topic from the
    /// component.
    pub fn is_event(&self, topic_name: &str) -> bool {
        topic_name.starts_with("logevent_")
    }

    /// Get topic info for a particular command.
    fn get_command_topic_info(&self, topic_name: &str) -> Option<&TopicInfo> {
        self.component_info.get_topic_info_command(topic_name)
    }

    /// Get topic info for a particular event.
    fn get_event_topic_info(&self, topic_name: &str) -> Option<&TopicInfo> {
        self.component_info.get_topic_info_event(topic_name)
    }

    /// Get topic info for a particular telemetry.
    fn get_telemetry_topic_info(&self, topic_name: &str) -> Option<&TopicInfo> {
        self.component_info.get_topic_info_telemetry(topic_name)
    }

    /// Get schema for topic.
    pub fn get_topic_schema(&self, topic_name: &str) -> Option<&Schema> {
        self.topic_schema.get(topic_name)
    }

    /// Assert that a topic name is a valid topic for this component.
    ///
    /// # Panic
    ///
    /// If topic name is not part of the component.
    pub fn assert_is_valid_topic(&self, topic_name: &str) {
        assert!(
            self.topic_schema.contains_key(topic_name) || topic_name == "ackcmd",
            "No topic {} in component {}:: Valid topics are {:?}",
            topic_name,
            self.get_name(),
            self.topic_schema.keys()
        )
    }

    pub async fn encode(
        &self,
        data_fields: Vec<(&str, Value)>,
        key_strategy: SubjectNameStrategy,
    ) -> Result<Vec<u8>, SRCError> {
        self.encoder.encode(data_fields, key_strategy).await
    }

    pub async fn decode(&self, bytes: Option<&[u8]>) -> Result<DecodeResult, SRCError> {
        self.decoder.decode(bytes).await
    }

    pub async fn register_schema(&self) -> HashMap<String, Result<RegisteredSchema, SRCError>> {
        let sr_settings = SalInfo::make_sr_settings();
        let mut result: HashMap<String, Result<RegisteredSchema, SRCError>> = HashMap::new();

        let topic_schema = self.component_info.make_avro_schema();

        for (topic, avro_schema) in topic_schema.iter() {
            let schema = serde_json::to_string(&avro_schema).unwrap();
            let supplied_schema = SuppliedSchema {
                name: Some(self.make_schema_registry_topic_name(topic)),
                schema_type: SchemaType::Avro,
                schema,
                references: vec![],
            };
            let decode =
                post_schema(&sr_settings, self.make_subject_name(topic), supplied_schema).await;
            result.entry(topic.to_owned()).or_insert(decode);
        }
        result
    }

    pub fn make_sr_settings() -> SrSettings {
        SrSettings::new(Domain::get_schema_registry_url())
    }

    pub fn make_encoder<'b>() -> AvroEncoder<'b> {
        let sr_settings = SalInfo::make_sr_settings();
        AvroEncoder::new(sr_settings)
    }

    pub fn make_decoder<'b>() -> AvroDecoder<'b> {
        let sr_settings = SalInfo::make_sr_settings();
        AvroDecoder::new(sr_settings)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sal_info_get_command_names() {
        let sal_info = SalInfo::new("Test", 1);

        let command_names = sal_info.get_command_names();

        assert!(command_names.contains(&"command_setScalars".to_owned()))
    }

    #[test]
    fn sal_info_get_event_names() {
        let sal_info = SalInfo::new("Test", 1);

        let event_names = sal_info.get_event_names();

        assert!(event_names.contains(&"logevent_scalars".to_owned()))
    }

    #[test]
    fn sal_info_get_telemetry_names() {
        let sal_info = SalInfo::new("Test", 1);

        let telemetry_names = sal_info.get_telemetry_names();

        assert!(telemetry_names.contains(&"scalars".to_owned()))
    }

    #[test]
    #[should_panic(expected = "Invalid index=1. Component ATMCS is not indexed. Index must be 0.")]
    fn panic_if_index_for_non_indexed() {
        let _ = SalInfo::new("ATMCS", 1);
    }

    #[test]
    fn get_name_index_indexed() {
        let sal_info = SalInfo::new("Test", 1);

        assert_eq!(sal_info.get_name_index(), "Test:1")
    }

    #[test]
    fn get_name_index_non_indexed() {
        let sal_info = SalInfo::new("ATMCS", 0);

        assert_eq!(sal_info.get_name_index(), "ATMCS")
    }

    #[test]
    fn make_ackcmd() {
        let sal_info = SalInfo::new("Test", 1);

        let ackcmd = sal_info.make_ackcmd(
            12345,
            sal_enums::SalRetCode::CmdComplete,
            0,
            "Command completed successfully.",
            60.0,
        );

        let fields: HashMap<String, Value> = ackcmd.fields.into_iter().collect();

        let private_seqnum = match fields.get("private_seqNum").unwrap() {
            Value::Int(value) => value.to_owned(),
            _ => panic!("wrong type for private_seqNum."),
        };
        let ack = match fields.get("ack").unwrap() {
            Value::Int(value) => value.to_owned(),
            _ => panic!("wrong type for ack."),
        };
        let result = match fields.get("result").unwrap() {
            Value::String(value) => value.to_owned(),
            _ => panic!("wrong type for result."),
        };
        let timeout = match fields.get("timeout").unwrap() {
            Value::Float(value) => value.to_owned(),
            _ => panic!("wrong type for timeout."),
        };

        assert_eq!(private_seqnum, 12345 as i32);
        assert_eq!(ack, sal_enums::SalRetCode::CmdComplete as i32);
        assert_eq!(result, "Command completed successfully.");
        assert_eq!(timeout, 60.0);
    }

    #[test]
    fn assert_is_valid_topic_with_valid_topic() {
        let sal_info = SalInfo::new("Test", 1);

        sal_info.assert_is_valid_topic("logevent_scalars")
    }

    #[test]
    #[should_panic]
    fn assert_is_valid_topic_with_invalid_topic() {
        let sal_info = SalInfo::new("Test", 1);

        sal_info.assert_is_valid_topic("logevent_badTopicName")
    }

    #[test]
    fn get_topic_info_ackcmd() {
        let sal_info = SalInfo::new("Test", 1);

        // This will panic if fails to get ackcmd
        sal_info.get_topic_info(&"ackcmd").unwrap();
    }

    #[test]
    fn get_topic_info_command() {
        let sal_info = SalInfo::new("Test", 1);

        // This will panic if fails to get command
        sal_info.get_topic_info(&"command_start").unwrap();
    }

    #[test]
    #[should_panic]
    fn get_topic_info_bad_command() {
        let sal_info = SalInfo::new("Test", 1);

        // This will panic if fails to get command
        sal_info.get_topic_info(&"command_startBad").unwrap();
    }

    #[test]
    fn get_topic_info_event_scalars() {
        let sal_info = SalInfo::new("Test", 1);

        // This will panic if fails to get event
        sal_info.get_topic_info(&"logevent_scalars").unwrap();
    }

    #[test]
    #[should_panic]
    fn get_topic_info_bad_event() {
        let sal_info = SalInfo::new("Test", 1);

        // This will panic if fails to get event
        sal_info.get_topic_info(&"logevent_scalarsBad").unwrap();
    }

    #[test]
    fn get_topic_info_telemetry() {
        let sal_info = SalInfo::new("Test", 1);

        // This will panic if fails to get telemetry
        sal_info.get_topic_info(&"scalars").unwrap();
    }

    #[test]
    #[should_panic]
    fn get_topic_info_bad_telemetry() {
        let sal_info = SalInfo::new("Test", 1);

        // This will panic if fails to get telemetry
        sal_info.get_topic_info(&"scalarsBad").unwrap();
    }
}
