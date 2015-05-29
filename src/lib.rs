#![allow(unused_unsafe)]

extern crate libc;
extern crate sqlite3_sys as raw;

use libc::c_int;
use std::path::Path;

/// A result.
pub type Result<T> = std::result::Result<T, Error>;

/// An error.
#[derive(Debug)]
pub struct Error {
    pub code: ResultCode,
    pub message: Option<String>,
}

/// A result code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultCode {
    Abort = raw::SQLITE_ABORT as isize,
    Authorization = raw::SQLITE_AUTH as isize,
    Busy = raw::SQLITE_BUSY as isize,
    CantOpen = raw::SQLITE_CANTOPEN as isize,
    Constraint = raw::SQLITE_CONSTRAINT as isize,
    Corruption = raw::SQLITE_CORRUPT as isize,
    Done = raw::SQLITE_DONE as isize,
    Empty = raw::SQLITE_EMPTY as isize,
    Error = raw::SQLITE_ERROR as isize,
    Format = raw::SQLITE_FORMAT as isize,
    Full = raw::SQLITE_FULL as isize,
    Internal = raw::SQLITE_INTERNAL as isize,
    Interruption = raw::SQLITE_INTERRUPT as isize,
    IOError = raw::SQLITE_IOERR as isize,
    Locked = raw::SQLITE_LOCKED as isize,
    Mismatch = raw::SQLITE_MISMATCH as isize,
    Misuse = raw::SQLITE_MISUSE as isize,
    NoLargeFileSupport = raw::SQLITE_NOLFS as isize,
    NoMemory = raw::SQLITE_NOMEM as isize,
    NotDatabase = raw::SQLITE_NOTADB as isize,
    NotFound = raw::SQLITE_NOTFOUND as isize,
    Notice = raw::SQLITE_NOTICE as isize,
    OK = raw::SQLITE_OK as isize,
    Permission = raw::SQLITE_PERM as isize,
    Protocol = raw::SQLITE_PROTOCOL as isize,
    Range = raw::SQLITE_RANGE as isize,
    ReadOnly = raw::SQLITE_READONLY as isize,
    Row = raw::SQLITE_ROW as isize,
    Schema = raw::SQLITE_SCHEMA as isize,
    TooBig = raw::SQLITE_TOOBIG as isize,
    Warning = raw::SQLITE_WARNING as isize,
}

impl ResultCode {
    #[inline]
    fn from_raw(code: c_int) -> ResultCode {
        unsafe { std::mem::transmute(code as i8) }
    }
}

macro_rules! raise(
    ($message:expr) => (
        return Err(::Error { code: ::ResultCode::Error, message: Some($message.to_string()) })
    );
    ($code:expr, $message:expr) => (
        return Err(::Error { code: $code, message: $message })
    );
);

macro_rules! success(
    ($result:expr) => (
        match $result {
            ::raw::SQLITE_OK => {},
            code => raise!(::ResultCode::from_raw(code), None),
        }
    );
);

macro_rules! path_to_c_str(
    ($path:expr) => ({
        match $path.to_str() {
            Some(path) => match ::std::ffi::CString::new(path) {
                Ok(string) => string.as_ptr(),
                Err(_) => raise!("failed to process a path"),
            },
            None => raise!("failed to process a path"),
        }
    });
);

macro_rules! str_to_c_str(
    ($string:expr) => (
        match ::std::ffi::CString::new($string) {
            Ok(string) => string.as_ptr(),
            Err(_) => raise!("failed to process a string"),
        }
    );
);

macro_rules! c_str_to_string(
    ($cstr:expr) => (
        String::from_utf8_lossy(::std::ffi::CStr::from_ptr($cstr as *const _).to_bytes())
               .into_owned()
    );
);

mod database;
mod statement;

pub use database::{Database, ExecuteCallback};
pub use statement::{Statement, Binding};

/// Open a database.
#[inline]
pub fn open(path: &Path) -> Result<Database> {
    Database::open(path)
}
