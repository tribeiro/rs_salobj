//! # Object-Oriented Service Abstraction Layer (SAL).
//!
//! This library provides an implementation of [salobj][1] in rust.
//!
//! SALObj is a framework used to develop control software for the Vera
//! Rubin Observatory. The implementation here is based in the original Python
//! kafka implementation of the library.
//!
//! This library is still under heavy development and is, at the time of this
//! writing, a rust learning exercise. Maybe in a distant future it will be
//! adopted by the project.
//!
//! An interesting note, in Spanish and Portuguese "sal" means "salt", which
//! is known to make metal "rust".
//!
//! # Version history
//!
//! ## 0.1.0
//!
//! - Initial implementation of the library.
//!
//! [1]: https://ts-salobj.lsst.io

#[macro_use]
extern crate serde_derive;

mod component_info;
pub mod controller;
pub mod csc;
pub mod domain;
pub mod error;
pub mod generics;
pub mod remote;
pub mod sal_enums;
pub mod sal_info;
mod sal_subsystem;
pub mod topics;
pub mod utils;

// Lets the `::salobj::...` paths emitted by `base_topic_derive` and
// `ts_xml_define` resolve when those macros are used inside salobj itself.
extern crate self as salobj;

pub use base_topic_derive::{add_sal_topic_fields, BaseSALTopic};

/// Re-exports used by code that salobj's macros generate.
///
/// Not public API: this lets generated code reach `chrono` and `serde`
/// through `::salobj`, so crates using the macros don't need their own
/// direct dependency on these crates.
#[doc(hidden)]
pub mod __private {
    pub use chrono;
    pub use serde;
    pub use serde_derive::{Deserialize, Serialize};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
