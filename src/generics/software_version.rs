use crate::{topics::base_sal_topic::BaseSALTopic, utils::xml_utils::get_default_sal_index};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};

#[add_sal_topic_fields]
#[derive(Debug, Deserialize, BaseSALTopic)]
pub struct SoftwareVersion {
    #[serde(rename = "salVersion")]
    sal_version: String,
    #[serde(rename = "xmlVersion")]
    xml_version: String,
    #[serde(rename = "openSpliceVersion")]
    open_splice_version: String,
    #[serde(rename = "cscVersion")]
    csc_version: String,
    #[serde(rename = "subsystemVersions")]
    subsystem_versions: String,
}

impl SoftwareVersion {
    pub fn get_sal_version(&self) -> String {
        self.sal_version.to_owned()
    }
    pub fn get_xml_version(&self) -> String {
        self.xml_version.to_owned()
    }
    pub fn get_open_splice_version(&self) -> String {
        self.open_splice_version.to_owned()
    }
    pub fn get_csc_version(&self) -> String {
        self.csc_version.to_owned()
    }
    pub fn get_subsystem_versions(&self) -> String {
        self.subsystem_versions.to_owned()
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
            .get_topic_schemas()
            .into_iter()
            .map(|(name, schema)| (name.to_owned(), Schema::parse_str(&schema).unwrap()))
            .collect();

        let topic_schema = avro_schema.get("logevent_softwareVersions").unwrap();
        let mut topic_record = Record::new(&topic_schema).unwrap();

        topic_record.put("salVersion", Value::String("vX.Y.Z".to_owned()));
        topic_record.put("xmlVersion", Value::String("vX.Y.Z".to_owned()));
        topic_record.put("openSpliceVersion", Value::String("vX.Y.Z".to_owned()));
        topic_record.put("cscVersion", Value::String("vX.Y.Z".to_owned()));
        topic_record.put("subsystemVersions", Value::String("vX.Y.Z".to_owned()));

        topic_record.put(
            "private_sndStamp",
            Value::Union(0, Box::new(Value::Double(1.234))),
        );
        topic_record.put("private_origin", Value::Int(123));
        topic_record.put("private_identity", Value::String("unit@test".to_string()));
        topic_record.put("private_seqNum", Value::Int(321));
        topic_record.put(
            "private_rcvStamp",
            Value::Union(0, Box::new(Value::Double(4.321))),
        );
        topic_record.put("salIndex", Value::Int(1));
        topic_record.put(
            "private_efdStamp",
            Value::Union(0, Box::new(Value::Double(1.234))),
        );
        topic_record.put(
            "private_kafkaStamp",
            Value::Union(0, Box::new(Value::Double(1.234))),
        );
        topic_record.put("private_revCode", Value::String("xyz".to_string()));

        let mut writer = Writer::with_codec(&topic_schema, Vec::new(), Codec::Deflate);
        writer.append(topic_record).unwrap();

        let input = writer.into_inner().unwrap();
        let reader = Reader::with_schema(&topic_schema, &input[..]).unwrap();

        for record in reader {
            let topic = from_value::<SoftwareVersion>(&record.unwrap()).unwrap();

            assert_eq!(topic.get_sal_version(), "vX.Y.Z");
            assert_eq!(topic.get_xml_version(), "vX.Y.Z");
            assert_eq!(topic.get_open_splice_version(), "vX.Y.Z");
            assert_eq!(topic.get_csc_version(), "vX.Y.Z");
            assert_eq!(topic.get_subsystem_versions(), "vX.Y.Z");

            assert_eq!(topic.get_private_origin(), 123);
            assert_eq!(topic.get_private_identity(), "unit@test".to_string());
            assert_eq!(topic.get_private_seq_num(), 321);
            assert_eq!(topic.get_private_rcv_stamp(), 4.321);
            assert_eq!(topic.get_sal_index(), 1);
        }
    }
}
