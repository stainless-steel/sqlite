use crate::sqlite3_connector as ffi;
use crate::{Result, Statement};

use std::marker::PhantomData;
use std::path::Path;

/// A database connection.
pub struct Connection {
    raw: ffi::Sqlite3DbHandle,
    phantom: PhantomData<ffi::Sqlite3DbHandle>,
}

/// Flags for opening a database connection.
#[derive(Clone, Copy, Debug)]
pub struct OpenFlags(i32);

unsafe impl Send for Connection {}

impl Connection {
    /// Open a read-write connection to a new or existing database.
    pub fn open<T: AsRef<Path>>(path: T) -> Result<Connection> {
        Connection::open_with_flags(path, OpenFlags::new().set_create().set_read_write())
    }

    /// Open a database connection with specific flags.
    pub fn open_with_flags<T: AsRef<Path>>(path: T, flags: OpenFlags) -> Result<Connection> {
        unsafe {
            let path = path.as_ref();
            let path = path.to_string_lossy().into_owned();
            let result = ffi::sqlite3_open_v2(&path, flags.0, &String::new());

            match result.ret_code {
                ffi::SQLITE_OK => {}
                code => {
                    return match crate::last_error(result.db_handle) {
                        Some(error) => {
                            ffi::sqlite3_close(result.db_handle);
                            Err(error)
                        }
                        _ => {
                            ffi::sqlite3_close(result.db_handle);
                            Err(crate::Error {
                                code: Some(code as isize),
                                message: None,
                            })
                        }
                    }
                }
            }

            Ok(Connection {
                raw: result.db_handle,
                phantom: PhantomData,
            })
        }
    }

    /// Execute a statement without processing the resulting rows if any.
    #[inline]
    pub fn execute<T: AsRef<str>>(&self, statement: T) -> Result<()> {
        unsafe {
            ok_descr!(
                self.raw,
                ffi::sqlite3_exec(self.raw, statement.as_ref().into(), 0, 0,)
            );
        }
        Ok(())
    }

    /// Execute a statement and process the resulting rows as plain text.
    ///
    /// The callback is triggered for each row. If the callback returns `false`,
    /// no more rows will be processed. For large queries and non-string data
    /// types, prepared statement are highly preferable; see `prepare`.
    #[inline]
    pub fn iterate<T: AsRef<str>, F>(&self, statement: T, callback: F) -> Result<()>
    where
        F: FnMut(&[(&str, Option<&str>)]) -> bool,
    {
        unsafe {
            let _callback = Box::new(callback);
            ok_descr!(
                self.raw,
                ffi::sqlite3_exec(self.raw, statement.as_ref().into(), 0, 0,)
            );
        }
        Ok(())
    }

    /// Create a prepared statement.
    #[inline]
    pub fn prepare<T: AsRef<str>>(&self, statement: T) -> Result<Statement> {
        crate::statement::new(self.raw, statement)
    }

    /// Return the number of rows inserted, updated, or deleted by the most
    /// recent INSERT, UPDATE, or DELETE statement.
    #[inline]
    pub fn changes(&self) -> usize {
        unsafe { ffi::sqlite3_changes(self.raw) as usize }
    }

    /// Return the total number of rows inserted, updated, and deleted by all
    /// INSERT, UPDATE, and DELETE statements since the connection was opened.
    #[inline]
    pub fn total_changes(&self) -> usize {
        unsafe { ffi::sqlite3_total_changes(self.raw) as usize }
    }

    /// Set an implicit callback for handling busy events that tries to repeat
    /// rejected operations until a timeout expires.
    #[inline]
    pub fn set_busy_timeout(&mut self, milliseconds: usize) -> Result<()> {
        unsafe {
            ok_raw!(
                self.raw,
                ffi::sqlite3_busy_timeout(self.raw, milliseconds as _)
            );
        }
        Ok(())
    }

    /// Return the raw pointer.
    #[inline]
    pub fn as_raw(&self) -> ffi::Sqlite3DbHandle {
        self.raw
    }
}

impl Drop for Connection {
    #[inline]
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_close(self.raw) };
    }
}

impl OpenFlags {
    /// Create flags for opening a database connection.
    #[inline]
    pub fn new() -> Self {
        OpenFlags(0)
    }

    /// Create the database if it does not already exist.
    pub fn set_create(mut self) -> Self {
        self.0 |= ffi::SQLITE_OPEN_CREATE;
        self
    }

    /// Open the database in the serialized [threading mode][1].
    ///
    /// [1]: https://www.sqlite.org/threadsafe.html
    pub fn set_full_mutex(mut self) -> Self {
        self.0 |= ffi::SQLITE_OPEN_FULLMUTEX;
        self
    }

    /// Opens the database in the multi-thread [threading mode][1].
    ///
    /// [1]: https://www.sqlite.org/threadsafe.html
    pub fn set_no_mutex(mut self) -> Self {
        self.0 |= ffi::SQLITE_OPEN_NOMUTEX;
        self
    }

    /// Open the database for reading only.
    pub fn set_read_only(mut self) -> Self {
        self.0 |= ffi::SQLITE_OPEN_READONLY;
        self
    }

    /// Open the database for reading and writing.
    pub fn set_read_write(mut self) -> Self {
        self.0 |= ffi::SQLITE_OPEN_READWRITE;
        self
    }
}

/*
extern "C" fn process_callback<F>(
    callback: *mut c_void,
    count: i32,
    values: *mut *mut c_char,
    columns: *mut *mut c_char,
) -> i32
where
    F: FnMut(&[(&str, Option<&str>)]) -> bool,
{
    unsafe {
        let mut pairs = Vec::with_capacity(count as usize);
        for i in 0..(count as isize) {
            let column = {
                let pointer = *columns.offset(i);
                debug_assert!(!pointer.is_null());
                c_str_to_str!(pointer).unwrap()
            };
            let value = {
                let pointer = *values.offset(i);
                if pointer.is_null() {
                    None
                } else {
                    Some(c_str_to_str!(pointer).unwrap())
                }
            };
            pairs.push((column, value));
        }
        if (*(callback as *mut F))(&pairs) {
            0
        } else {
            1
        }
    }
}
*/
