#[macro_export]
macro_rules! base_topic {
    ($t:ident) => {
        impl Topic for $t {
            fn get_private_origin(&self) -> i64 {
                self.private_origin
            }
            fn get_private_identity(&self) -> String {
                self.private_identity.to_owned()
            }
            fn get_private_seq_num(&self) -> i64 {
                self.private_seq_num
            }
            fn get_private_rcv_stamp(&self) -> f64 {
                self.private_rcv_stamp
            }
            fn get_sal_index(&self) -> i64 {
                self.sal_index
            }
        }
    };
}
