//! Interface to [SQLite][1].
//!
//! ## Example
//!
//! Open a connection, create a table, and insert a few rows:
//!
//! ```
//! let connection = sqlite::open(":memory:").unwrap();
//!
//! let query = "
//!     CREATE TABLE users (name TEXT, age INTEGER);
//!     INSERT INTO users VALUES ('Alice', 42);
//!     INSERT INTO users VALUES ('Bob', 69);
//! ";
//! connection.execute(query).unwrap();
//! ```
//!
//! Select some rows and process them one by one as plain text, which is generally
//! not efficient:
//!
//! ```
//! # let connection = sqlite::open(":memory:").unwrap();
//! # let query = "
//! #     CREATE TABLE users (name TEXT, age INTEGER);
//! #     INSERT INTO users VALUES ('Alice', 42);
//! #     INSERT INTO users VALUES ('Bob', 69);
//! # ";
//! # connection.execute(query).unwrap();
//! let query = "SELECT * FROM users WHERE age > 50";
//!
//! connection
//!     .iterate(query, |pairs| {
//!         for &(name, value) in pairs.iter() {
//!             println!("{} = {}", name, value.unwrap());
//!         }
//!         true
//!     })
//!     .unwrap();
//! ```
//!
//! Run the same query but using a prepared statement, which is much more efficient
//! than the previous technique:
//!
//! ```
//! use sqlite::State;
//! # let connection = sqlite::open(":memory:").unwrap();
//! # let query = "
//! #     CREATE TABLE users (name TEXT, age INTEGER);
//! #     INSERT INTO users VALUES ('Alice', 42);
//! #     INSERT INTO users VALUES ('Bob', 69);
//! # ";
//! # connection.execute(query).unwrap();
//!
//! let query = "SELECT * FROM users WHERE age > ?";
//! let mut statement = connection.prepare(query).unwrap();
//! statement.bind((1, 50)).unwrap();
//!
//! while let Ok(State::Row) = statement.next() {
//!     println!("name = {}", statement.read::<String, _>("name").unwrap());
//!     println!("age = {}", statement.read::<i64, _>("age").unwrap());
//! }
//! ```
//!
//! Run the same query but using a cursor, which is iterable:
//!
//! ```
//! # let connection = sqlite::open(":memory:").unwrap();
//! # let query = "
//! #     CREATE TABLE users (name TEXT, age INTEGER);
//! #     INSERT INTO users VALUES ('Alice', 42);
//! #     INSERT INTO users VALUES ('Bob', 69);
//! # ";
//! # connection.execute(query).unwrap();
//!
//! let query = "SELECT * FROM users WHERE age > ?";
//!
//! for row in connection
//!     .prepare(query)
//!     .unwrap()
//!     .into_iter()
//!     .bind((1, 50))
//!     .unwrap()
//!     .map(|row| row.unwrap())
//! {
//!     println!("name = {}", row.read::<&str, _>("name"));
//!     println!("age = {}", row.read::<i64, _>("age"));
//! }
//! ```
//!
//! [1]: https://www.sqlite.org

extern crate sqlite3_sys as ffi;

macro_rules! c_str_to_str(
    ($string:expr) => (std::str::from_utf8(std::ffi::CStr::from_ptr($string).to_bytes()));
);

macro_rules! c_str_to_string(
    ($string:expr) => (
        String::from_utf8_lossy(std::ffi::CStr::from_ptr($string as *const _).to_bytes())
               .into_owned()
    );
);

macro_rules! path_to_cstr(
    ($path:expr) => (
        match $path.to_str() {
            Some(path) => {
                match std::ffi::CString::new(path) {
                    Ok(string) => string,
                    _ => raise!("failed to process a path"),
                }
            }
            _ => raise!("failed to process a path"),
        }
    );
);

macro_rules! str_to_cstr(
    ($string:expr) => (
        match std::ffi::CString::new($string) {
            Ok(string) => string,
            _ => raise!("failed to process a string"),
        }
    );
);

#[macro_use]
mod error;
mod value;

mod connection;
mod cursor;
mod statement;

pub use error::{Error, Result};
pub use value::{Type, Value};

pub use connection::{Connection, ConnectionThreadSafe, OpenFlags};
pub use cursor::{Cursor, CursorWithOwnership, Row, RowIndex};
pub use statement::{
    Bindable, BindableWithIndex, ColumnIndex, ParameterIndex, ReadableWithIndex, State, Statement,
};

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
