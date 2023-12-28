use crate::{topics::base_sal_topic::BaseSALTopic, utils::xml_utils::get_default_sal_index};
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};

#[add_sal_topic_fields]
#[derive(Debug, Deserialize, Serialize, BaseSALTopic, Clone)]
pub struct Arrays {
    pub boolean0: Vec<bool>,
    pub byte0: Vec<u8>,
    pub short0: Vec<i16>,
    pub int0: Vec<i32>,
    pub long0: Vec<i32>,
    #[serde(rename = "longLong0")]
    pub long_long0: Vec<i64>,
    #[serde(rename = "unsignedShort0")]
    pub unsigned_short0: Vec<u16>,
    #[serde(rename = "unsignedInt0")]
    pub unsigned_int0: Vec<u64>,
    pub float0: Vec<f32>,
    pub double0: Vec<f64>,
    // pub string0: String,
}

impl Default for Arrays {
    fn default() -> Self {
        Self {
            boolean0: vec![false, false, false, false, false],
            byte0: vec![0, 0, 0, 0, 0],
            short0: vec![0, 0, 0, 0, 0],
            int0: vec![0, 0, 0, 0, 0],
            long0: vec![0, 0, 0, 0, 0],
            long_long0: vec![0, 0, 0, 0, 0],
            unsigned_short0: vec![0, 0, 0, 0, 0],
            unsigned_int0: vec![0, 0, 0, 0, 0],
            float0: vec![0.0, 0.0, 0.0, 0.0, 0.0],
            double0: vec![0.0, 0.0, 0.0, 0.0, 0.0],
            // string0: "".to_owned(),
            private_origin: 0,
            private_identity: "".to_owned(),
            private_seq_num: 0,
            private_rcv_stamp: 0.0,
            private_snd_stamp: 0.0,
            sal_index: 0,
            private_efd_stamp: 0.0,
            private_kafka_stamp: 0.0,
            private_rev_code: "".to_owned(),
        }
    }
}
