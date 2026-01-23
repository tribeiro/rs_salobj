//! Base topic definition.

use apache_avro::types::Record;

/// A trait that represents base topic interface.
pub trait BaseTopic {
    /// String with the record type used in kafka.
    fn get_record_type(&self) -> String {
        "value".to_owned()
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
