use ResultCode;

use std::convert::{From, Into};

/// An error.
#[derive(Debug)]
pub struct Error {
    pub code: ResultCode,
    pub message: Option<String>,
}

impl<T> From<T> for Error where T: Into<String> {
    #[inline]
    fn from(message: T) -> Error {
        Error {
            code: ResultCode::Error,
            message: Some(message.into()),
        }
    }
}

impl From<ResultCode> for Error {
    #[inline]
    fn from(code: ResultCode) -> Error {
        Error {
            code: code,
            message: None,
        }
    }
}
