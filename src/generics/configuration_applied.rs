use crate::{base_topic, topics::topic::Topic, utils::xml_utils::get_default_sal_index};

#[derive(Debug, Deserialize)]
pub struct ConfigurationApplied {
    configurations: String,
    version: String,
    url: String,
    #[serde(rename = "schemaVersion")]
    schema_version: String,
    #[serde(rename = "otherInfo")]
    other_info: String,
    private_origin: i64,
    private_identity: String,
    #[serde(rename = "private_seqNum")]
    private_seq_num: i64,
    #[serde(rename = "private_rcvStamp")]
    private_rcv_stamp: f64,
    #[serde(rename = "salIndex", default = "get_default_sal_index")]
    sal_index: i64,
}

base_topic!(ConfigurationApplied);

impl ConfigurationApplied {
    pub fn get_configurations(&self) -> String {
        self.configurations.to_owned()
    }
    pub fn get_version(&self) -> String {
        self.version.to_owned()
    }
    pub fn get_url(&self) -> String {
        self.url.to_owned()
    }
    pub fn get_schema_version(&self) -> String {
        self.schema_version.to_owned()
    }
    pub fn get_other_info(&self) -> String {
        self.other_info.to_owned()
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
        let component_info = ComponentInfo::new("Test", "unit_test");

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

        let topic_schema = avro_schema.get("logevent_configurationApplied").unwrap();
        let mut topic_record = Record::new(&topic_schema).unwrap();

        topic_record.put(
            "configurations",
            Value::String("configurations".to_string()),
        );
        topic_record.put("version", Value::String("version".to_string()));
        topic_record.put("url", Value::String("url".to_string()));
        topic_record.put("schemaVersion", Value::String("schema_version".to_string()));
        topic_record.put("otherInfo", Value::String("other_info".to_string()));
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
            let topic = from_value::<ConfigurationApplied>(&record.unwrap()).unwrap();

            assert_eq!(topic.get_configurations(), "configurations");
            assert_eq!(topic.get_version(), "version");
            assert_eq!(topic.get_url(), "url");
            assert_eq!(topic.get_schema_version(), "schema_version");
            assert_eq!(topic.get_other_info(), "other_info");
            assert_eq!(topic.get_private_origin(), 123);
            assert_eq!(topic.get_private_identity(), "unit@test".to_string());
            assert_eq!(topic.get_private_seq_num(), 321);
            assert_eq!(topic.get_private_rcv_stamp(), 4.321);
            assert_eq!(topic.get_sal_index(), 1);
        }
    }
}
