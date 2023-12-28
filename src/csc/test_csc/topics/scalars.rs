use crate::{
    topics::base_sal_topic::BaseSALTopic,
    utils::{xml_utils::get_default_sal_index},
};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};

#[add_sal_topic_fields]
#[derive(Debug, Deserialize, Serialize, BaseSALTopic, Default, Clone)]
pub struct Scalars {
    pub boolean0: bool,
    pub byte0: u8,
    pub short0: i16,
    pub int0: i32,
    pub long0: i32,
    #[serde(rename = "longLong0")]
    pub long_long0: i64,
    #[serde(rename = "unsignedShort0")]
    pub unsigned_short0: u16,
    #[serde(rename = "unsignedInt0")]
    pub unsigned_int0: u32,
    pub float0: f32,
    pub double0: f64,
    pub string0: String,
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
            .get_topic_schemas()
            .into_iter()
            .map(|(name, schema)| (name.to_owned(), Schema::parse_str(&schema).unwrap()))
            .collect();

        let topic_schema = avro_schema.get("logevent_scalars").unwrap();
        let mut topic_record = Record::new(&topic_schema).unwrap();

        topic_record.put("salIndex", Value::Int(3));
        topic_record.put(
            "private_sndStamp",
            Value::Union(0, Box::new(Value::Double(1700679476.9876108))),
        );
        topic_record.put(
            "private_rcvStamp",
            Value::Union(0, Box::new(Value::Double(0.0))),
        );
        topic_record.put(
            "private_efdStamp",
            Value::Union(0, Box::new(Value::Double(1700679476.9876108))),
        );
        topic_record.put(
            "private_kafkaStamp",
            Value::Union(0, Box::new(Value::Double(1700679476.9876108))),
        );
        topic_record.put("private_seqNum", Value::Int(2));
        topic_record.put("private_revCode", Value::String("Not Set".to_owned()));
        topic_record.put(
            "private_identity",
            Value::String("tribeiro@lynx.local".to_owned()),
        );
        topic_record.put("private_origin", Value::Int(13784));
        topic_record.put("boolean0", Value::Boolean(false));
        topic_record.put("byte0", Value::Int(0));
        topic_record.put("short0", Value::Int(0));
        topic_record.put("int0", Value::Int(10));
        topic_record.put("long0", Value::Int(0));
        topic_record.put("longLong0", Value::Int(0));
        topic_record.put("unsignedShort0", Value::Int(0));
        topic_record.put("unsignedInt0", Value::Int(0));
        topic_record.put(
            "float0",
            Value::Union(0, Box::new(Value::Float(0.33333334))),
        );
        topic_record.put(
            "double0",
            Value::Union(0, Box::new(Value::Double(0.3333333333333333))),
        );
        topic_record.put("string0", Value::String("this is a test".to_owned()));

        // topic_record.put("boolean0", Value::Boolean(true));
        // topic_record.put("byte0", Value::Int(1));
        // topic_record.put("short0", Value::Int(1));
        // topic_record.put("int0", Value::Int(1));
        // topic_record.put("long0", Value::Int(1));
        // topic_record.put("longLong0", Value::Int(1));
        // topic_record.put("unsignedShort0", Value::Int(1));
        // topic_record.put("unsignedInt0", Value::Int(1));
        // topic_record.put("float0", Value::Union(0, Box::new(Value::Float(1.0))));
        // topic_record.put("double0", Value::Union(0, Box::new(Value::Double(1.0))));
        // topic_record.put("string0", Value::String("This is a test".to_owned()));

        // topic_record.put("private_sndStamp", Value::Union(0, Box::new(Value::Double(1.234))));
        // topic_record.put("private_origin", Value::Int(123));
        // topic_record.put("private_identity", Value::String("unit@test".to_string()));
        // topic_record.put("private_seqNum", Value::Int(321));
        // topic_record.put("private_rcvStamp", Value::Union(0, Box::new(Value::Double(4.321))));
        // topic_record.put("salIndex", Value::Int(1));
        // topic_record.put("private_efdStamp", Value::Union(0, Box::new(Value::Double(1.234))));
        // topic_record.put("private_kafkaStamp", Value::Union(0, Box::new(Value::Double(1.234))));
        // topic_record.put("private_revCode", Value::String("xyz".to_string()));

        let mut writer = Writer::with_codec(&topic_schema, Vec::new(), Codec::Deflate);
        writer.append(topic_record).unwrap();

        let input = writer.into_inner().unwrap();
        let reader = Reader::with_schema(&topic_schema, &input[..]).unwrap();

        for record in reader {
            let topic = from_value::<Scalars>(&record.unwrap()).unwrap();

            assert_eq!(topic.get_sal_index(), 3);
            assert_eq!(topic.get_private_rcv_stamp(), 0.0);
            assert_eq!(topic.get_private_seq_num(), 2);
            assert_eq!(
                topic.get_private_identity(),
                "tribeiro@lynx.local".to_owned()
            );
            assert_eq!(topic.get_private_origin(), 13784);
            assert_eq!(topic.boolean0, false);
            assert_eq!(topic.byte0, 0);
            assert_eq!(topic.short0, 0);
            assert_eq!(topic.int0, 10);
            assert_eq!(topic.long0, 0);
            assert_eq!(topic.long_long0, 0);
            assert_eq!(topic.unsigned_short0, 0);
            assert_eq!(topic.unsigned_int0, 0);
            assert_eq!(topic.float0, 0.33333334);
            assert_eq!(topic.double0, 0.3333333333333333);
            assert_eq!(topic.string0, "this is a test".to_owned());
        }
    }
}
