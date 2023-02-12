#[macro_export]
/// Implement [BaseTopic](crate::topics::base_topic::BaseTopic) trait to a
/// struct.
///
/// If the struct adheres to the SAL
/// [base topic definition](crate::generics), you only have to
/// do:
///
/// ```
/// use salobj::{base_topic, topics::topic::Topic, utils::xml_utils::get_default_sal_index};
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize)]
/// pub struct ExampleBaseTopic {
///     private_origin: i64,
///     private_identity: String,
///     #[serde(rename = "private_seqNum")]
///     private_seq_num: i64,
///     #[serde(rename = "private_rcvStamp")]
///     private_rcv_stamp: f64,
///     #[serde(rename = "salIndex", default = "get_default_sal_index")]
///     sal_index: i64,
/// }
///
/// base_topic!(ExampleBaseTopic);
/// ```
///
/// to implement it.
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
