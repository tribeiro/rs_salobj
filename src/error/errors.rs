use std::{error::Error, fmt, result};

pub type SalObjResult<T> = result::Result<T, SalObjError>;

#[derive(Debug)]
pub struct SalObjError {
    err_msg: String,
}

impl fmt::Display for SalObjError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = self.err_msg.clone();
        write!(f, "NoInterfaceFileError::{err_msg}")
    }
}

impl Error for SalObjError {}

impl SalObjError {
    pub fn new(err_msg: &str) -> SalObjError {
        SalObjError {
            err_msg: String::from(err_msg),
        }
    }

    pub fn from_error(error: impl Error) -> SalObjError {
        SalObjError {
            err_msg: error.to_string(),
        }
    }

    pub fn get_error_message(&self) -> &str {
        &self.err_msg
    }
}
