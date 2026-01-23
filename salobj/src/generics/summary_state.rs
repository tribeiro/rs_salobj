use crate::{
    sal_enums::State, topics::base_sal_topic::BaseSALTopic, utils::xml_utils::get_default_sal_index,
};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};
use chrono::Utc;

#[add_sal_topic_fields]
#[derive(Debug, Default, Deserialize, Serialize, BaseSALTopic)]
pub struct SummaryState {
    #[serde(rename = "summaryState")]
    summary_state: i32,
}

impl SummaryState {
    pub fn get_summary_state_value(&self) -> i32 {
        self.summary_state
    }

    pub fn get_summary_state(&self) -> State {
        State::from_summary_state(self)
    }

    pub fn with_summary_state(mut self, value: State) -> Self {
        self.summary_state = value.to::<i32>().unwrap_or(0);
        self
    }
    pub fn set_summary_state(&mut self, value: State) {
        self.summary_state = value.to::<i32>().unwrap_or(0);
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

        let summary_state_schema = avro_schema.get("logevent_summaryState").unwrap();
        let mut summary_state_record = Record::new(&summary_state_schema).unwrap();

        summary_state_record.put("summaryState", Value::Int(2));
        summary_state_record.put("private_sndStamp", Value::Double(1.234));
        summary_state_record.put("private_origin", Value::Int(123));
        summary_state_record.put("private_identity", Value::String("unit@test".to_string()));
        summary_state_record.put("private_seqNum", Value::Int(321));
        summary_state_record.put("private_rcvStamp", Value::Double(4.321));
        summary_state_record.put("salIndex", Value::Int(1));
        summary_state_record.put("private_efdStamp", Value::Double(1.234));
        summary_state_record.put("private_kafkaStamp", Value::Double(1.234));
        summary_state_record.put("private_revCode", Value::String("xyz".to_string()));

        let mut writer = Writer::with_codec(
            &summary_state_schema,
            Vec::new(),
            Codec::Deflate(DeflateSettings::new(
                miniz_oxide::deflate::CompressionLevel::NoCompression,
            )),
        );
        writer.append(summary_state_record).unwrap();

        let input = writer.into_inner().unwrap();
        let reader = Reader::with_schema(&summary_state_schema, &input[..]).unwrap();

        for record in reader {
            let summary_state = from_value::<SummaryState>(&record.unwrap()).unwrap();

            assert_eq!(summary_state.get_summary_state(), State::Enabled);
            assert_eq!(summary_state.get_private_origin(), 123);
            assert_eq!(
                summary_state.get_private_identity(),
                "unit@test".to_string()
            );
            assert_eq!(summary_state.get_private_seq_num(), 321);
            assert_eq!(summary_state.get_private_rcv_stamp(), 4.321);
            assert_eq!(summary_state.get_sal_index(), 1);
        }
    }
}
