pub trait BaseSALTopic {
    fn get_name(&self) -> &'static str;
    fn field_names(&self) -> Vec<&'static str>;
    fn get_private_origin(&self) -> i64;
    fn get_private_identity(&self) -> &str;
    fn get_private_seq_num(&self) -> i64;
    fn get_private_rcv_stamp(&self) -> f64;
    fn get_sal_index(&self) -> i64;
    fn set_private_snd_stamp(&mut self, value: f64);
    fn set_private_efd_stamp(&mut self, value: f64);
    fn set_private_kafka_stamp(&mut self, value: f64);
    fn set_private_origin(&mut self, value: i64);
    fn set_private_identity(&mut self, value: &str);
    fn set_private_rev_code(&mut self, value: &str);
    fn set_private_seq_num(&mut self, value: i64);
    fn set_private_rcv_stamp(&mut self, value: f64);
    fn set_sal_index(&mut self, value: i64);
}
