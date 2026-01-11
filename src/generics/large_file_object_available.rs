use crate::{topics::base_sal_topic::BaseSALTopic, utils::xml_utils::get_default_sal_index};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};
use chrono::Utc;

#[add_sal_topic_fields]
#[derive(Debug, Deserialize, BaseSALTopic)]
pub struct LargeFileObjectAvailable {
    url: String,
    generator: String,
    version: f32,
    #[serde(rename = "byteSize")]
    byte_size: i64,
    #[serde(rename = "checkSum")]
    check_sum: String,
    #[serde(rename = "mimeType")]
    mime_type: String,
    id: String,
}

impl LargeFileObjectAvailable {
    pub fn get_url(&self) -> String {
        self.url.to_owned()
    }
    pub fn get_generator(&self) -> String {
        self.generator.to_owned()
    }
    pub fn get_version(&self) -> f32 {
        self.version
    }
    pub fn get_byte_size(&self) -> i64 {
        self.byte_size
    }
    pub fn get_check_sum(&self) -> String {
        self.check_sum.to_owned()
    }
    pub fn get_mime_type(&self) -> String {
        self.mime_type.to_owned()
    }
    pub fn get_id(&self) -> String {
        self.id.to_owned()
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
        let component_info = ComponentInfo::new("Script", "unit_test").unwrap();

        let avro_schema: HashMap<String, Schema> = component_info
            .get_topic_schemas()
            .into_iter()
            .map(|(name, schema)| (name.to_owned(), Schema::parse_str(&schema).unwrap()))
            .collect();

        let topic_schema = avro_schema
            .get("logevent_largeFileObjectAvailable")
            .unwrap();
        let mut topic_record = Record::new(&topic_schema).unwrap();

        topic_record.put("url", Value::String("url".to_owned()));
        topic_record.put("generator", Value::String("generator".to_owned()));
        topic_record.put("version", Value::Float(1.0));
        topic_record.put("byteSize", Value::Int(123));
        topic_record.put("checkSum", Value::String("checkSum".to_owned()));
        topic_record.put("mimeType", Value::String("mimeType".to_owned()));
        topic_record.put("id", Value::String("id".to_owned()));

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
            &topic_schema,
            Vec::new(),
            Codec::Deflate(DeflateSettings::new(
                miniz_oxide::deflate::CompressionLevel::NoCompression,
            )),
        );
        writer.append(topic_record).unwrap();

        let input = writer.into_inner().unwrap();
        let reader = Reader::with_schema(&topic_schema, &input[..]).unwrap();

        for record in reader {
            let topic = from_value::<LargeFileObjectAvailable>(&record.unwrap()).unwrap();

            assert_eq!(topic.get_url(), "url");
            assert_eq!(topic.get_generator(), "generator");
            assert_eq!(topic.get_version(), 1.0);
            assert_eq!(topic.get_byte_size(), 123);
            assert_eq!(topic.get_check_sum(), "checkSum");
            assert_eq!(topic.get_mime_type(), "mimeType");
            assert_eq!(topic.get_id(), "id");

            assert_eq!(topic.get_private_origin(), 123);
            assert_eq!(topic.get_private_identity(), "unit@test".to_string());
            assert_eq!(topic.get_private_seq_num(), 321);
            assert_eq!(topic.get_private_rcv_stamp(), 4.321);
            assert_eq!(topic.get_sal_index(), 1);
        }
    }
}
