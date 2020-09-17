//! Interface to [SQLite][1].
//!
//! ## Example
//!
//! Open a connection, create a table, and insert some rows:
//!
//! ```
//! let connection = sqlite::open(":memory:").unwrap();
//!
//! connection
//!     .execute(
//!         "
//!         CREATE TABLE users (name TEXT, age INTEGER);
//!         INSERT INTO users VALUES ('Alice', 42);
//!         INSERT INTO users VALUES ('Bob', 69);
//!         ",
//!     )
//!     .unwrap();
//! ```
//!
//! Select some rows and process them one by one as plain text:
//!
//! ```
//! # let connection = sqlite::open(":memory:").unwrap();
//! # connection
//! #     .execute(
//! #         "
//! #         CREATE TABLE users (name TEXT, age INTEGER);
//! #         INSERT INTO users VALUES ('Alice', 42);
//! #         INSERT INTO users VALUES ('Bob', 69);
//! #         ",
//! #     )
//! #     .unwrap();
//! connection
//!     .iterate("SELECT * FROM users WHERE age > 50", |pairs| {
//!         for &(column, value) in pairs.iter() {
//!             println!("{} = {}", column, value.unwrap());
//!         }
//!         true
//!     })
//!     .unwrap();
//! ```
//!
//! The same query using a prepared statement, which is much more efficient than
//! the previous technique:
//!
//! ```
//! use sqlite::State;
//! # let connection = sqlite::open(":memory:").unwrap();
//! # connection
//! #     .execute(
//! #         "
//! #         CREATE TABLE users (name TEXT, age INTEGER);
//! #         INSERT INTO users VALUES ('Alice', 42);
//! #         INSERT INTO users VALUES ('Bob', 69);
//! #         ",
//! #     )
//! #     .unwrap();
//!
//! let mut statement = connection
//!     .prepare("SELECT * FROM users WHERE age > ?")
//!     .unwrap();
//!
//! statement.bind(1, 50).unwrap();
//!
//! while let State::Row = statement.next().unwrap() {
//!     println!("name = {}", statement.read::<String>(0).unwrap());
//!     println!("age = {}", statement.read::<i64>(1).unwrap());
//! }
//! ```
//!
//! The same query using a cursor, which is a wrapper around a prepared
//! statement providing the concept of row and featuring all-at-once binding:
//!
//! ```
//! use sqlite::Value;
//! # let connection = sqlite::open(":memory:").unwrap();
//! # connection
//! #     .execute(
//! #         "
//! #         CREATE TABLE users (name TEXT, age INTEGER);
//! #         INSERT INTO users VALUES ('Alice', 42);
//! #         INSERT INTO users VALUES ('Bob', 69);
//! #         ",
//! #     )
//! #     .unwrap();
//!
//! let mut cursor = connection
//!     .prepare("SELECT * FROM users WHERE age > ?")
//!     .unwrap()
//!     .cursor();
//!
//! cursor.bind(&[Value::Integer(50)]).unwrap();
//!
//! while let Some(row) = cursor.next().unwrap() {
//!     println!("name = {}", row[0].as_string().unwrap());
//!     println!("age = {}", row[1].as_integer().unwrap());
//! }
//! ```
//!
//! [1]: https://www.sqlite.org

#![allow(dead_code)]

use sqlite3_connector as ffi;

use std::{error, fmt};

macro_rules! error(
    ($connection:expr, $code:expr) => (
        match ::last_error($connection) {
            Some(error) => return Err(error),
            _ => return Err(::Error {
                code: Some($code as isize),
                message: None,
            }),
        }
    );
);

macro_rules! ok_descr(
    ($connection:expr, $result:expr) => (
        match $result.ret_code {
            ::ffi::SQLITE_OK => {}
            code => error!($connection, code),
        }
    );
    ($result:expr) => (
        match $result.ret_code {
            ::ffi::SQLITE_OK => {}
            code => return Err(::Error {
                code: Some(code as isize),
                message: None,
            }),
        }
    );
);

macro_rules! ok_raw(
    ($connection:expr, $result:expr) => (
        match $result {
            ::ffi::SQLITE_OK => {}
            code => error!($connection, code),
        }
    );
    ($result:expr) => (
        match $result {
            ::ffi::SQLITE_OK => {}
            code => return Err(::Error {
                code: Some(code as isize),
                message: None,
            }),
        }
    );
);

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

/// A data type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    /// The binary type.
    Binary,
    /// The floating-point type.
    Float,
    /// The integer type.
    Integer,
    /// The string type.
    String,
    /// The null type.
    Null,
}

/// A typed value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Binary data.
    Binary(Vec<u8>),
    /// A floating-point number.
    Float(f64),
    /// An integer number.
    Integer(i64),
    /// A string.
    String(String),
    /// A null value.
    Null,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match (self.code, &self.message) {
            (Some(code), &Some(ref message)) => write!(formatter, "{} (code {})", message, code),
            (Some(code), _) => write!(formatter, "an SQLite error (code {})", code),
            (_, &Some(ref message)) => message.fmt(formatter),
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

impl Value {
    /// Return the binary data if the value is `Binary`.
    #[inline]
    pub fn as_binary(&self) -> Option<&[u8]> {
        if let &Value::Binary(ref value) = self {
            return Some(value);
        }
        None
    }

    /// Return the floating-point number if the value is `Float`.
    #[inline]
    pub fn as_float(&self) -> Option<f64> {
        if let &Value::Float(value) = self {
            return Some(value);
        }
        None
    }

    /// Return the integer number if the value is `Integer`.
    #[inline]
    pub fn as_integer(&self) -> Option<i64> {
        if let &Value::Integer(value) = self {
            return Some(value);
        }
        None
    }

    /// Return the string if the value is `String`.
    #[inline]
    pub fn as_string(&self) -> Option<&str> {
        if let &Value::String(ref value) = self {
            return Some(value);
        }
        None
    }

    /// Return the type.
    pub fn kind(&self) -> Type {
        match self {
            &Value::Binary(_) => Type::Binary,
            &Value::Float(_) => Type::Float,
            &Value::Integer(_) => Type::Integer,
            &Value::String(_) => Type::String,
            &Value::Null => Type::Null,
        }
    }
}

mod connection;
mod cursor;
mod sqlite3_connector;
mod statement;

pub use connection::Connection;
pub use connection::OpenFlags;
pub use cursor::Cursor;
pub use statement::{Bindable, Readable, State, Statement};

/// Open a read-write connection to a new or existing database.
#[inline]
pub fn open<T: AsRef<std::path::Path>>(path: T) -> Result<Connection> {
    Connection::open(path)
}

/// Return the version number of SQLite.
///
/// For instance, the version `3.8.11.1` corresponds to the integer `3008011`.
#[inline]
pub fn version() -> usize {
    unsafe { ffi::sqlite3_libversion_number() as usize }
}

fn last_error(raw: ffi::Sqlite3DbHandle) -> Option<Error> {
    unsafe {
        let code = ffi::sqlite3_errcode(raw);
        if code == ffi::SQLITE_OK {
            return None;
        }
        let message = ffi::sqlite3_errmsg(raw);
        Some(Error {
            code: Some(code as isize),
            message: Some(message),
        })
    }
}
