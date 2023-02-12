//! Define structs for all generic topics.
//!
//! Generic topics are common topics shared by all SAL components. Most of
//! these topics are opt-in, although some are mandatory. Having these topics
//! defined as structs allow us to use [serde] to serialize/deserialize the
//! topics directly into structs.
//!
//! All topics must implement the
//! [BaseTopic](crate::topics::base_topic::BaseTopic) trait. This can be done
//! with the [base_topic](crate::base_topic) macro, so long as the struct adhere to the
//! [base topic](crate::topics::base_topic) definition.
//!
//! At a minimum a base topic struct will have the following fields:
//!
//! ```
//! use salobj::utils::xml_utils::get_default_sal_index;
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! pub struct ExampleBaseTopic {
//!     private_origin: i64,
//!     private_identity: String,
//!     #[serde(rename = "private_seqNum")]
//!     private_seq_num: i64,
//!     #[serde(rename = "private_rcvStamp")]
//!     private_rcv_stamp: f64,
//!     #[serde(rename = "salIndex", default = "get_default_sal_index")]
//!     sal_index: i64,
//! }
//! ```

pub mod auth_list;
pub mod configuration_applied;
pub mod configurations_available;
pub mod disable;
pub mod enable;
pub mod enter_control;
pub mod error_code;
pub mod heartbeat;
pub mod large_file_object_available;
pub mod log_level;
pub mod log_message;
pub mod set_auth_list;
pub mod set_log_level;
pub mod simulation_mode;
pub mod software_version;
pub mod standby;
pub mod start;
pub mod status_code;
pub mod summary_state;
