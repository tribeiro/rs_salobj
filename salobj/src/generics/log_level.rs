use crate::{topics::base_sal_topic::BaseSALTopic, utils::xml_utils::get_default_sal_index};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};
use chrono::Utc;

#[add_sal_topic_fields]
#[derive(Debug, Deserialize, BaseSALTopic)]
pub struct LogLevel {
    level: i32,
    subsystem: String,
}

impl LogLevel {
    pub fn get_level(&self) -> i32 {
        self.level
    }
    pub fn get_subsystem(&self) -> String {
        self.subsystem.to_owned()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::component_info::ComponentInfo;
    use apache_avro::from_value;
    use apache_avro::DeflateSettings;
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
            .get_topic_schemas()
            .into_iter()
            .map(|(name, schema)| (name.to_owned(), Schema::parse_str(&schema).unwrap()))
            .collect();

        let topic_schema = avro_schema.get("logevent_logLevel").unwrap();
        let mut topic_record = Record::new(topic_schema).unwrap();

        topic_record.put("level", Value::Int(0));
        topic_record.put("subsystem", Value::String("subsystem".to_string()));
        topic_record.put("private_sndStamp", Value::Double(1.234));
        topic_record.put("private_origin", Value::Int(123));
        topic_record.put("private_identity", Value::String("unit@test".to_string()));
        topic_record.put("private_seqNum", Value::Int(321));
        topic_record.put("private_rcvStamp", Value::Double(4.321));
        topic_record.put("salIndex", Value::Int(1));
        topic_record.put("private_efdStamp", Value::Double(1.234));
        topic_record.put("private_kafkaStamp", Value::Double(1.234));
        topic_record.put("private_revCode", Value::String("xyz".to_string()));

        let mut writer = Writer::with_codec(
            topic_schema,
            Vec::new(),
            Codec::Deflate(DeflateSettings::new(
                miniz_oxide::deflate::CompressionLevel::NoCompression,
            )),
        );
        writer.append(topic_record).unwrap();

        let input = writer.into_inner().unwrap();
        let reader = Reader::with_schema(topic_schema, &input[..]).unwrap();

        for record in reader {
            let topic = from_value::<LogLevel>(&record.unwrap()).unwrap();

            assert_eq!(topic.get_level(), 0);
            assert_eq!(topic.get_subsystem(), "subsystem");
            assert_eq!(topic.get_private_origin(), 123);
            assert_eq!(topic.get_private_identity(), "unit@test".to_string());
            assert_eq!(topic.get_private_seq_num(), 321);
            assert_eq!(topic.get_private_rcv_stamp(), 4.321);
            assert_eq!(topic.get_sal_index(), 1);
        }
    }
}
