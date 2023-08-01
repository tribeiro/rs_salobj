use crate::{topics::base_sal_topic::BaseSALTopic, utils::xml_utils::get_default_sal_index};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};

#[add_sal_topic_fields]
#[derive(Debug, Deserialize, BaseSALTopic)]
pub struct Standby {}

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

        let topic_schema = avro_schema.get("command_standby").unwrap();
        let mut topic_record = Record::new(&topic_schema).unwrap();

        topic_record.put("private_sndStamp", Value::Double(1.234));
        topic_record.put("private_origin", Value::Long(123));
        topic_record.put("private_identity", Value::String("unit@test".to_string()));
        topic_record.put("private_seqNum", Value::Long(321));
        topic_record.put("private_rcvStamp", Value::Double(4.321));
        topic_record.put("salIndex", Value::Long(1));

        let mut writer = Writer::with_codec(&topic_schema, Vec::new(), Codec::Deflate);
        writer.append(topic_record).unwrap();

        let input = writer.into_inner().unwrap();
        let reader = Reader::with_schema(&topic_schema, &input[..]).unwrap();

        for record in reader {
            let topic = from_value::<Standby>(&record.unwrap()).unwrap();

            assert_eq!(topic.get_private_origin(), 123);
            assert_eq!(topic.get_private_identity(), "unit@test".to_string());
            assert_eq!(topic.get_private_seq_num(), 321);
            assert_eq!(topic.get_private_rcv_stamp(), 4.321);
            assert_eq!(topic.get_sal_index(), 1);
        }
    }
}
