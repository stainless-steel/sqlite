use raw;
use std::convert::{From, Into};

use {Database, ResultCode};

/// An error.
#[derive(Debug)]
pub struct Error {
    pub code: ResultCode,
    pub message: Option<String>,
}

impl Error {
    /// Return the last occurred error if any.
    pub fn last(database: &mut Database) -> Option<Error> {
        unsafe {
            let code = raw::sqlite3_errcode(::database::as_raw(database));
            if code == raw::SQLITE_OK {
                return None;
            }
            let message = raw::sqlite3_errmsg(::database::as_raw(database));
            if message.is_null() {
                return None;
            }
            Some(Error {
                code: ::result::code_from_raw(code),
                message: Some(c_str_to_string!(message)),
            })
        }
    }
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
