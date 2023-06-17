use std::{error, fmt};

/// An error.
#[derive(Debug)]
pub struct Error {
    /// The error code.
    pub code: Option<isize>,
    /// The error message.
    pub message: Option<String>,
}

/// A result.
pub type Result<T> = std::result::Result<T, Error>;

macro_rules! error(
    ($connection:expr, $code:expr) => (
        match crate::error::last($connection) {
            Some(error) => return Err(error),
            _ => return Err(crate::error::Error {
                code: Some($code as isize),
                message: None,
            }),
        }
    );
);

macro_rules! ok(
    ($connection:expr, $result:expr) => (
        match $result {
            crate::ffi::SQLITE_OK => {}
            code => error!($connection, code),
        }
    );
    ($result:expr) => (
        match $result {
            crate::ffi::SQLITE_OK => {}
            code => return Err(crate::error::Error {
                code: Some(code as isize),
                message: None,
            }),
        }
    );
);

macro_rules! raise(
    ($message:expr $(, $($token:tt)* )?) => (
        return Err(crate::error::Error {
            code: None,
            message: Some(format!($message $(, $($token)* )*)),
        })
    );
);

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match (self.code, &self.message) {
            (Some(code), Some(message)) => write!(formatter, "{message} (code {code})"),
            (Some(code), _) => write!(formatter, "an SQLite error (code {code})"),
            (_, Some(message)) => message.fmt(formatter),
            _ => write!(formatter, "an SQLite error"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self.message {
            Some(ref message) => message,
            _ => "an SQLite error",
        }
    }
}

pub fn last(raw: *mut ffi::sqlite3) -> Option<Error> {
    unsafe {
        let code = ffi::sqlite3_errcode(raw);
        if code == ffi::SQLITE_OK {
            return None;
        }
        let message = ffi::sqlite3_errmsg(raw);
        if message.is_null() {
            return None;
        }
        Some(Error {
            code: Some(code as isize),
            message: Some(c_str_to_string!(message)),
        })
    }
}
