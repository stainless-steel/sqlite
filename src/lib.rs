#![allow(unused_unsafe)]

extern crate libc;
extern crate sqlite3_sys as raw;

use std::path::Path;

/// A result.
pub type Result<T> = std::result::Result<T, Error>;

/// An error.
#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub message: Option<String>,
}

macro_rules! raise(
    ($message:expr) => (
        return Err(::Error { code: ::ErrorCode::Error, message: Some($message.to_string()) })
    );
    ($code:expr, $message:expr) => (
        return Err(::Error { code: $code, message: $message })
    );
);

macro_rules! success(
    ($result:expr) => (
        match $result {
            ::raw::SQLITE_OK => {},
            code => raise!(unsafe { ::std::mem::transmute(code as i8) }, None),
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

/// A result code.
#[derive(Clone, Copy, Debug)]
pub enum ErrorCode {
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

/// A database.
pub struct Database {
    db: *mut raw::sqlite3,
}

impl Database {
    pub fn open(path: &Path) -> Result<Database> {
        let mut db = 0 as *mut _;
        unsafe { success!(raw::sqlite3_open(path_to_c_str!(path), &mut db)) };
        Ok(Database { db: db })
    }
}

impl Drop for Database {
    #[inline]
    fn drop(&mut self) {
        unsafe { ::raw::sqlite3_close(self.db) };
    }
}

#[inline]
pub fn open(path: &Path) -> Result<Database> {
    Database::open(path)
}
