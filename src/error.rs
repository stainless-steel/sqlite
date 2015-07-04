use ffi;
use std::convert::{From, Into};
use std::{error, fmt};

/// An error.
#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: Option<String>,
}

macro_rules! declare(
    ($($left:ident => $right:ident,)*) => (
        /// An error kind.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum ErrorKind {
            $($left = ffi::$right as isize,)*
            Unknown,
        }

        impl From<isize> for ErrorKind {
            fn from(code: isize) -> ErrorKind {
                match code as ::libc::c_int {
                    $(ffi::$right => ErrorKind::$left,)*
                    _ => ErrorKind::Unknown,
                }
            }
        }
    );
);

declare!(
    Abort => SQLITE_ABORT,
    Authorization => SQLITE_AUTH,
    Busy => SQLITE_BUSY,
    CantOpen => SQLITE_CANTOPEN,
    Constraint => SQLITE_CONSTRAINT,
    Corruption => SQLITE_CORRUPT,
    Done => SQLITE_DONE,
    Empty => SQLITE_EMPTY,
    Error => SQLITE_ERROR,
    Format => SQLITE_FORMAT,
    Full => SQLITE_FULL,
    Internal => SQLITE_INTERNAL,
    Interruption => SQLITE_INTERRUPT,
    IOError => SQLITE_IOERR,
    Locked => SQLITE_LOCKED,
    Mismatch => SQLITE_MISMATCH,
    Misuse => SQLITE_MISUSE,
    NoLargeFileSupport => SQLITE_NOLFS,
    NoMemory => SQLITE_NOMEM,
    NotDatabase => SQLITE_NOTADB,
    NotFound => SQLITE_NOTFOUND,
    Notice => SQLITE_NOTICE,
    OK => SQLITE_OK,
    Permission => SQLITE_PERM,
    Protocol => SQLITE_PROTOCOL,
    Range => SQLITE_RANGE,
    ReadOnly => SQLITE_READONLY,
    Row => SQLITE_ROW,
    Schema => SQLITE_SCHEMA,
    TooBig => SQLITE_TOOBIG,
    Warning => SQLITE_WARNING,
);

impl<T> From<T> for Error where T: Into<String> {
    #[inline]
    fn from(message: T) -> Error {
        Error { kind: ErrorKind::Unknown, message: Some(message.into()) }
    }
}

impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error { kind: kind, message: None }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.message {
            Some(ref message) => message.fmt(formatter),
            _ => self.kind.fmt(formatter),
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

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::Unknown => write!(formatter, "an unknown SQLite result code"),
            _ => write!(formatter, "SQLite result code {}", *self as isize),
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
            kind: ErrorKind::from(code as isize),
            message: Some(c_str_to_string!(message)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorKind;

    #[test]
    fn fmt() {
        assert_eq!(format!("{}", ErrorKind::OK),
                   String::from("SQLite result code 0"));
        assert_eq!(format!("{}", ErrorKind::from(777)),
                   String::from("an unknown SQLite result code"));
    }
}
