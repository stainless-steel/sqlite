use ffi;
use libc::{c_char, c_int, c_void};
use std::marker::PhantomData;
use std::path::Path;

use {Result, Statement};

/// A database connection.
pub struct Connection {
    raw: *mut ffi::sqlite3,
    busy_callback: Option<Box<dyn FnMut(usize) -> bool>>,
    phantom: PhantomData<ffi::sqlite3>,
}

/// A set of connection flags.
#[derive(Clone, Copy, Debug)]
pub struct ConnectionFlags(c_int);

unsafe impl Send for Connection {}

impl Connection {
    /// Open a read-write connection to a new or existing database.
    pub fn open<T: AsRef<Path>>(path: T) -> Result<Connection> {
        Connection::open_with_flags(path, ConnectionFlags::new().set_create().set_read_write())
    }

    /// Open a database connection with a specific set of connection flags.
    pub fn open_with_flags<T: AsRef<Path>>(path: T, flags: ConnectionFlags) -> Result<Connection> {
        let mut raw = 0 as *mut _;
        unsafe {
            ok!(ffi::sqlite3_open_v2(
                path_to_cstr!(path.as_ref()).as_ptr(),
                &mut raw,
                flags.0,
                0 as *const _,
            ));
        }
        Ok(Connection {
            raw: raw,
            busy_callback: None,
            phantom: PhantomData,
        })
    }

    /// Execute a statement without processing the resulting rows if any.
    #[inline]
    pub fn execute<T: AsRef<str>>(&self, statement: T) -> Result<()> {
        unsafe {
            ok!(
                self.raw,
                ffi::sqlite3_exec(
                    self.raw,
                    str_to_cstr!(statement.as_ref()).as_ptr(),
                    None,
                    0 as *mut _,
                    0 as *mut _,
                )
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
            let callback = Box::new(callback);
            ok!(
                self.raw,
                ffi::sqlite3_exec(
                    self.raw,
                    str_to_cstr!(statement.as_ref()).as_ptr(),
                    Some(process_callback::<F>),
                    &*callback as *const F as *mut F as *mut _,
                    0 as *mut _,
                )
            );
        }
        Ok(())
    }

    /// Create a prepared statement.
    #[inline]
    pub fn prepare<'l, T: AsRef<str>>(&'l self, statement: T) -> Result<Statement<'l>> {
        ::statement::new(self.raw, statement)
    }

    /// Set a callback for handling busy events.
    ///
    /// The callback is triggered when the database cannot perform an operation
    /// due to processing of some other request. If the callback returns `true`,
    /// the operation will be repeated.
    pub fn set_busy_handler<F>(&mut self, callback: F) -> Result<()>
    where
        F: FnMut(usize) -> bool + Send + 'static,
    {
        try!(self.remove_busy_handler());
        unsafe {
            let callback = Box::new(callback);
            let result = ffi::sqlite3_busy_handler(
                self.raw,
                Some(busy_callback::<F>),
                &*callback as *const F as *mut F as *mut _,
            );
            self.busy_callback = Some(callback);
            ok!(self.raw, result);
        }
        Ok(())
    }

    /// Set an implicit callback for handling busy events that tries to repeat
    /// rejected operations until a timeout expires.
    #[inline]
    pub fn set_busy_timeout(&mut self, milliseconds: usize) -> Result<()> {
        unsafe {
            ok!(
                self.raw,
                ffi::sqlite3_busy_timeout(self.raw, milliseconds as c_int)
            );
        }
        Ok(())
    }

    /// Remove the callback handling busy events.
    #[inline]
    pub fn remove_busy_handler(&mut self) -> Result<()> {
        self.busy_callback = None;
        unsafe {
            ok!(
                self.raw,
                ffi::sqlite3_busy_handler(self.raw, None, 0 as *mut _)
            );
        }
        Ok(())
    }

    /// Return the raw pointer.
    #[inline]
    pub fn as_raw(&self) -> *mut ffi::sqlite3 {
        self.raw
    }
}

impl Drop for Connection {
    #[inline]
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.remove_busy_handler();
        unsafe { ffi::sqlite3_close(self.raw) };
    }
}

impl ConnectionFlags {
    /// Create a set of connection flags.
    #[inline]
    pub fn new() -> Self {
        ConnectionFlags(0)
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

extern "C" fn busy_callback<F>(callback: *mut c_void, attempts: c_int) -> c_int
where
    F: FnMut(usize) -> bool,
{
    unsafe {
        if (*(callback as *mut F))(attempts as usize) {
            1
        } else {
            0
        }
    }
}

extern "C" fn process_callback<F>(
    callback: *mut c_void,
    count: c_int,
    values: *mut *mut c_char,
    columns: *mut *mut c_char,
) -> c_int
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
