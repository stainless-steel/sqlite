#![allow(unused_unsafe)]

extern crate libc;
extern crate sqlite3_sys as raw;

use libc::{c_char, c_int, c_void};
use std::marker::PhantomData;
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

macro_rules! str_to_c_str(
    ($string:expr) => (
        match ::std::ffi::CString::new($string) {
            Ok(string) => string.as_ptr(),
            Err(_) => raise!("failed to process a string"),
        }
    );
);

/// An error code.
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
pub struct Database<'d> {
    db: *mut raw::sqlite3,
    _phantom: PhantomData<&'d raw::sqlite3>,
}

struct ExecuteCallback<'d>(Box<FnMut(Vec<(String, String)>) -> bool + 'd>);

impl<'d> Database<'d> {
    /// Open a database.
    pub fn open(path: &Path) -> Result<Database<'d>> {
        let mut db = 0 as *mut _;
        unsafe {
            success!(raw::sqlite3_open(path_to_c_str!(path), &mut db));
        }
        Ok(Database { db: db, _phantom: PhantomData })
    }

    /// Execute an SQL statement.
    pub fn execute<F>(&mut self, sql: &str, callback: Option<F>) -> Result<()>
        where F: FnMut(Vec<(String, String)>) -> bool {

        unsafe {
            match callback {
                Some(callback) => {
                    let mut callback = ExecuteCallback(Box::new(callback));
                    success!(raw::sqlite3_exec(self.db, str_to_c_str!(sql), Some(execute_callback),
                                               &mut callback as *mut _ as *mut _, 0 as *mut _));
                },
                None => {
                    success!(raw::sqlite3_exec(self.db, str_to_c_str!(sql), None,
                                               0 as *mut _, 0 as *mut _));
                },
            }
        }

        Ok(())
    }
}

impl<'d> Drop for Database<'d> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ::raw::sqlite3_close(self.db) };
    }
}

/// Open a database.
#[inline]
pub fn open(path: &Path) -> Result<Database> {
    Database::open(path)
}

extern fn execute_callback(callback: *mut c_void, count: c_int, values: *mut *mut c_char,
                           columns: *mut *mut c_char) -> c_int {

    macro_rules! c_str_to_string(
        ($string:expr) => (
            match ::std::str::from_utf8(::std::ffi::CStr::from_ptr($string).to_bytes()) {
                Ok(string) => String::from(string),
                Err(_) => return 1,
            }
        );
    );

    unsafe {
        let mut pairs = Vec::with_capacity(count as usize);

        for i in 0..(count as isize) {
            let column = c_str_to_string!(*columns.offset(i) as *const _);
            let value = c_str_to_string!(*values.offset(i) as *const _);
            pairs.push((column, value));
        }

        let ExecuteCallback(ref mut callback) = *(callback as *mut _);
        if callback(pairs) { 0 } else { 1 }
    }
}
