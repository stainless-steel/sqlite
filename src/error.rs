use raw;
use std::convert::{From, Into};
use std::fmt::{self, Display, Formatter};

use ResultCode;

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

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self.message {
            Some(ref message) => Display::fmt(message, formatter),
            None => Display::fmt(&self.code, formatter),
        }
    }
}

pub fn last(raw: *mut raw::sqlite3) -> Option<Error> {
    unsafe {
        let code = raw::sqlite3_errcode(raw);
        if code == raw::SQLITE_OK {
            return None;
        }
        let message = raw::sqlite3_errmsg(raw);
        if message.is_null() {
            return None;
        }
        Some(Error {
            code: ::result::code_from_raw(code),
            message: Some(c_str_to_string!(message)),
        })
    }
}
