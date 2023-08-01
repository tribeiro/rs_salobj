pub trait BaseSALTopic {
    fn get_name(&self) -> &'static str;
    fn field_names(&self) -> Vec<&'static str>;
    fn get_private_origin(&self) -> i64;
    fn get_private_identity(&self) -> &str;
    fn get_private_seq_num(&self) -> i64;
    fn get_private_rcv_stamp(&self) -> f64;
    fn get_sal_index(&self) -> i64;
}
