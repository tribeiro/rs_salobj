use crate::topics::topic_info::TopicInfo;
use avro_rs::{types::Record, Schema};

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
    fn get_avro_schema(&self) -> &Schema;

    /// Make data type.
    ///
    /// This method creates the topic record from the topic schema. Generating
    /// the topic involves parsing the topic avro schema and then creating a
    /// record, which can be slow to do every single time you want to generate
    /// a topic record. Instead, use this method when creating the topic and
    /// store a copy in your class, then use `get_data_type` to retrieve it.
    fn make_data_type(&self) -> Record {
        let record = Record::new(self.get_avro_schema()).unwrap().to_owned();

        record
    }

    /// Get data type.
    ///
    /// In principle this method should return the same value produced by
    /// `make_data_type`. But, instead of creating the topic record, this
    /// method can return a previously stored copy of the topic. When
    /// implementing the `BaseTopic` trait, use `make_data_type` to create the
    /// record and implement `get_data_type` to return it.
    fn get_data_type(&self) -> Record;
}
