//! Base topic definition.

use crate::sal_info::SalInfo;
use apache_avro::types::Record;

/// A trait that represents base topic interface.
pub trait BaseTopic {
    /// String with the record type used in kafka.
    fn get_record_type(&self) -> String {
        "value".to_owned()
    }

    /// Get avro schema.
    ///
    /// This method returns a borrow copy of the topic Schema. When implementing
    /// `BaseTopic`, use `make_avro_schema` to store the topic schema and this
    /// method to return it. This will make the code faster for runtime topic
    /// creation.
    fn get_avro_schema(sal_info: &SalInfo, topic_name: &str) -> Option<apache_avro::Schema> {
        sal_info.get_topic_schema(topic_name).cloned()
    }

    /// Make data type.
    ///
    /// This method creates the topic record from the topic schema. Generating
    /// the topic involves parsing the topic avro schema and then creating a
    /// record, which can be slow to do every single time you want to generate
    /// a topic record. Instead, use this method when creating the topic and
    /// store a copy in your class, then use `get_data_type` to retrieve it.
    fn make_data_type(avro_schema: &apache_avro::Schema) -> Option<Record> {
        Record::new(avro_schema)
    }
}
