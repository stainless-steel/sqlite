use libc::c_int;
use std::fmt::{self, Display, Formatter};

use raw;
use Error;

/// A result.
pub type Result<T> = ::std::result::Result<T, Error>;

macro_rules! declare(
    ($($left:ident => $right:ident,)*) => (
        /// A result code.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum ResultCode {
            $($left = raw::$right as isize,)*
            Unknown,
        }

        pub fn code_from_raw(code: c_int) -> ResultCode {
            match code {
                $(raw::$right => ResultCode::$left,)*
                _ => ResultCode::Unknown,
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

impl Display for ResultCode {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            ResultCode::Unknown => write!(formatter, "an unknown SQLite code"),
            _ => write!(formatter, "SQLite code {}", *self as isize),
        }
    }
}

#[cfg(test)]
mod tests {
    use ResultCode;
    use super::code_from_raw;

    #[test]
    fn fmt() {
        assert_eq!(format!("{}", ResultCode::OK), String::from("SQLite code 0"));
        assert_eq!(format!("{}", code_from_raw(777)), String::from("an unknown SQLite code"));
    }
}
