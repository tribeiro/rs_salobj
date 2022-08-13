use crate::{
    sal_info::SalInfo,
    topics::{base_topic::BaseTopic, topic_info::TopicInfo},
};
use std::rc::Rc;

/// Base struct for reading a topic.
pub struct ReadTopic {
    sal_info: Rc<SalInfo>,
    sal_name: String,
}

impl ReadTopic {
    pub fn new(sal_info: Rc<SalInfo>, sal_name: &str) -> ReadTopic {
        sal_info.assert_is_valid_topic(sal_name);

        ReadTopic {
            sal_info: sal_info,
            sal_name: sal_name.to_owned(),
        }
    }
}

impl BaseTopic for ReadTopic {
    fn get_topic_info(&self) -> &TopicInfo {
        self.sal_info.get_topic_info(&self.sal_name).unwrap()
    }

    fn get_avro_schema(&self) -> &avro_rs::Schema {
        &self.sal_info.get_topic_schema(&self.sal_name).unwrap()
    }

    fn get_data_type(&self) -> avro_rs::types::Record {
        self.make_data_type()
    }
}
