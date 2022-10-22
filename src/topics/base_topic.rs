use crate::{sal_info::SalInfo, topics::topic_info::TopicInfo};
use avro_rs::types::Record;

/// A trait that represents base topic interface.
pub trait BaseTopic {
    /// Return reference to TopicInfo.
    ///
    /// Any implementation of BaseTopic must contain an instance of TopicInfo.
    fn get_topic_info(&self) -> &TopicInfo;

    /// Return name of the topic with the standard SAL format.
    ///
    /// Events have the format, `logevent_<event_name>`, telemetry are simply
    /// `<telemetry_name>` and commands, `command_<command_name>`.
    fn get_sal_name(&self) -> String {
        self.get_topic_info().get_sal_name()
    }

    /// Get avro schema.
    ///
    /// This method returns a borrow copy of the topic Schema. When implementing
    /// `BaseTopic`, use `make_avro_schema` to store the topic schema and this
    /// method to return it. This will make the code faster for runtime topic
    /// creation.
    fn get_avro_schema(sal_info: &SalInfo, sal_name: &str) -> avro_rs::Schema {
        sal_info.get_topic_schema(sal_name).unwrap().clone()
    }

    /// Make data type.
    ///
    /// This method creates the topic record from the topic schema. Generating
    /// the topic involves parsing the topic avro schema and then creating a
    /// record, which can be slow to do every single time you want to generate
    /// a topic record. Instead, use this method when creating the topic and
    /// store a copy in your class, then use `get_data_type` to retrieve it.
    fn make_data_type(avro_schema: &avro_rs::Schema) -> Record {
        let record = Record::new(avro_schema).unwrap().to_owned();

        record
    }
}
