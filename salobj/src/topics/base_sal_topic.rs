pub trait BaseSALTopic {
    fn get_name(&self) -> &'static str;
    fn field_names(&self) -> Vec<&'static str>;
    fn get_private_origin(&self) -> i32;
    fn get_private_identity(&self) -> &str;
    fn get_private_seq_num(&self) -> i32;
    fn get_private_rcv_stamp(&self) -> f64;
    fn get_sal_index(&self) -> i32;
    fn with_timestamps(self) -> Self;
    fn with_private_origin(self, value: i32) -> Self;
    fn with_private_identity(self, value: &str) -> Self;
    fn with_private_rev_code(self, value: &str) -> Self;
    fn with_private_seq_num(self, value: i32) -> Self;
    fn with_sal_index(self, value: i32) -> Self;
}
