#[macro_use]
extern crate serde_derive;

mod component_info;
mod domain;
mod remote;
mod sal_info;
mod sal_subsystem;
mod topics;
mod utils;
mod error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
