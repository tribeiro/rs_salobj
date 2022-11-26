#[macro_export]
macro_rules! base_topic {
    ($t:ident) => {
        impl<'a> BaseTopic for $t<'a> {
            fn get_topic_info(&self) -> &TopicInfo {
                self.sal_info.get_topic_info(&self.topic_name).unwrap()
            }

            fn get_sal_name(&self) -> String {
                self.sal_info.get_sal_name(&self.topic_name)
            }

            fn get_topic_publish_name(&self) -> String {
                self.sal_info.make_topic_name(&self.topic_name)
            }

            fn get_record_type(&self) -> String {
                "value".to_owned()
            }
        }
    };
}
