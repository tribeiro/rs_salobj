use crate::utils::xml_utils::get_default_sal_index;
use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};

#[add_sal_topic_fields(indexed)]
#[derive(Debug, Deserialize, Default, Serialize, BaseSALTopic, Clone)]
pub struct Wait {
    pub ack: i32,
    pub duration: f64,
}
