use crate::topics::field_info;
use crate::topics::sal_objects::Item;
use std::collections::HashMap;

/// Information about one topic.
pub struct TopicInfo {
    component_name: String,
    topic_subname: String,
    sal_name: String,
    fields: HashMap<String, field_info::FieldInfo>,
    description: String,
    partitions: usize,
}

impl TopicInfo {
    pub fn new() -> TopicInfo {
        TopicInfo {
            component_name: String::new(),
            topic_subname: String::new(),
            sal_name: String::new(),
            fields: HashMap::new(),
            description: String::new(),
            partitions: 0,
        }
    }
    pub fn from_items(items: &Vec<Item>) -> TopicInfo {
        TopicInfo {
            component_name: String::new(),
            topic_subname: String::new(),
            sal_name: String::new(),
            fields: HashMap::new(),
            description: String::new(),
            partitions: 0,
        }
    }
}
