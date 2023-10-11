use crate::{topics::base_sal_topic::BaseSALTopic, utils::xml_utils::get_default_sal_index};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};

#[add_sal_topic_fields]
#[derive(Debug, Deserialize, BaseSALTopic)]
pub struct LogMessage {
    name: String,
    level: isize,
    message: String,
    traceback: String,
    #[serde(rename = "filePath")]
    file_path: String,
    #[serde(rename = "functionName")]
    function_name: String,
    #[serde(rename = "lineNumber")]
    line_number: i64,
    process: i64,
    timestamp: f64,
}

impl LogMessage {
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }
    pub fn get_level(&self) -> isize {
        self.level
    }
    pub fn get_message(&self) -> String {
        self.message.to_owned()
    }
    pub fn get_traceback(&self) -> String {
        self.traceback.to_owned()
    }
    pub fn get_file_path(&self) -> String {
        self.file_path.to_owned()
    }
    pub fn get_function_name(&self) -> String {
        self.function_name.to_owned()
    }
    pub fn get_line_number(&self) -> i64 {
        self.line_number
    }
    pub fn get_process(&self) -> i64 {
        self.process
    }
    pub fn get_timestamp(&self) -> f64 {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::component_info::ComponentInfo;
    use apache_avro::from_value;
    use apache_avro::Reader;
    use apache_avro::{
        types::{Record, Value},
        Codec, Schema, Writer,
    };
    use std::collections::HashMap;

    #[test]
    fn test_deserialize() {
        let component_info = ComponentInfo::new("Test", "unit_test").unwrap();

        let avro_schema: HashMap<String, Schema> = component_info
            .make_avro_schema()
            .into_iter()
            .map(|(name, schema)| {
                (
                    name.to_owned(),
                    Schema::parse_str(&serde_json::to_string(&schema).unwrap()).unwrap(),
                )
            })
            .collect();

        let schema = avro_schema.get("logevent_logMessage").unwrap();
        let mut record = Record::new(&schema).unwrap();

        record.put("name", Value::String("Test".to_owned()));
        record.put("level", Value::Int(10));
        record.put("message", Value::String("test".to_owned()));
        record.put("traceback", Value::String("".to_owned()));
        record.put("filePath", Value::String("some_file".to_owned()));
        record.put("functionName", Value::String("some_function".to_owned()));
        record.put("lineNumber", Value::Long(123));
        record.put("process", Value::Long(321));
        record.put("timestamp", Value::Double(5e3));

        record.put("private_sndStamp", Value::Double(1.234));
        record.put("private_origin", Value::Long(123));
        record.put("private_identity", Value::String("unit@test".to_string()));
        record.put("private_seqNum", Value::Long(321));
        record.put("private_rcvStamp", Value::Double(4.321));
        record.put("salIndex", Value::Long(1));
        record.put("private_efdStamp", Value::Double(1.234));
        record.put("private_kafkaStamp", Value::Double(1.234));
        record.put("private_revCode", Value::String("xyz".to_string()));

        let mut writer = Writer::with_codec(&schema, Vec::new(), Codec::Deflate);
        writer.append(record).unwrap();

        let input = writer.into_inner().unwrap();
        let reader = Reader::with_schema(&schema, &input[..]).unwrap();

        for record in reader {
            let log_message = from_value::<LogMessage>(&record.unwrap()).unwrap();

            assert_eq!(log_message.get_name(), "Test");
            assert_eq!(log_message.get_level(), 10);
            assert_eq!(log_message.get_message(), "test");
            assert_eq!(log_message.get_traceback(), "");
            assert_eq!(log_message.get_file_path(), "some_file");
            assert_eq!(log_message.get_function_name(), "some_function");
            assert_eq!(log_message.get_line_number(), 123);
            assert_eq!(log_message.get_process(), 321);
            assert_eq!(log_message.get_timestamp(), 5e3);
        }
    }
}
