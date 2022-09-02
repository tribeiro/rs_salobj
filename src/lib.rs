//! # Rust SAL Object.
//!
//! This library provides an implementation of [salobj][1] in rust.
//!
//! It provides a framework used to develop control software for the Vera
//! Rubin Observatory. The implementation here is based of the kafka-based
//! Python implementation.
//!
//! This library is still under heavy development and is, at the time of this
//! writing, a rust learning exercise. Maybe in a distant future it will be
//! adopted by the project.
//!
//! Some interesting notes. In Spanish and Portuguese "sal" means "salt", which
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
pub mod domain;
mod error;
mod remote;
mod sal_enums;
pub mod sal_info;
mod sal_subsystem;
pub mod topics;
mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
