use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct NoInterfaceFileError {
    err_msg: String,
}

impl fmt::Display for NoInterfaceFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = self.err_msg.clone();
        write!(f, "NoInterfaceFileError::{err_msg}")
    }
}

impl Error for NoInterfaceFileError {}

impl NoInterfaceFileError {
    pub fn new(err_msg: &str) -> NoInterfaceFileError {
        NoInterfaceFileError {
            err_msg: String::from(err_msg),
        }
    }
}
