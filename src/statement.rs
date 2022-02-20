use ffi;
use libc::{c_double, c_int};
use std::marker::PhantomData;

use {Cursor, Result, Type, Value};

// https://sqlite.org/c3ref/c_static.html
macro_rules! transient(
    () => (::std::mem::transmute(!0 as *const ::libc::c_void));
);

/// A prepared statement.
pub struct Statement<'l> {
    raw: (*mut ffi::sqlite3_stmt, *mut ffi::sqlite3),
    phantom: PhantomData<(ffi::sqlite3_stmt, &'l ffi::sqlite3)>,
}

/// A state of a prepared statement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// There is a row available for reading.
    Row,
    /// The statement has been entirely evaluated.
    Done,
}

/// A type suitable for binding to a prepared statement.
pub trait Bindable {
    /// Bind to a parameter.
    ///
    /// The first parameter has index 1.
    fn bind(self, _: &mut Statement, _: usize) -> Result<()>;
}

/// A type suitable for reading from a prepared statement.
pub trait Readable: Sized {
    /// Read from a column.
    ///
    /// The first column has index 0.
    fn read(_: &Statement, _: usize) -> Result<Self>;
}

impl<'l> Statement<'l> {
    /// Bind a value to a parameter by index.
    ///
    /// The first parameter has index 1.
    #[inline]
    pub fn bind<T: Bindable>(&mut self, i: usize, value: T) -> Result<()> {
        value.bind(self, i)
    }

    /// Bind a value to a parameter by name.
    ///
    /// # Examples
    ///
    /// ```
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (name STRING)");
    /// let mut statement = connection.prepare("SELECT * FROM users WHERE name = :name")?;
    /// statement.bind_by_name(":name", "Bob")?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    pub fn bind_by_name<T: Bindable>(&mut self, name: &str, value: T) -> Result<()> {
        if let Some(i) = self.parameter_index(name)? {
            self.bind(i, value)
        } else {
            raise!(format!("no such parameter: {}", name))
        }
    }

    /// Return the number of columns.
    #[inline]
    pub fn column_count(&self) -> usize {
        unsafe { ffi::sqlite3_column_count(self.raw.0) as usize }
    }

    /// Return the name of a column.
    ///
    /// The first column has index 0.
    #[inline]
    pub fn column_name(&self, i: usize) -> &str {
        debug_assert!(i < self.column_count(), "the index is out of range");
        unsafe {
            let pointer = ffi::sqlite3_column_name(self.raw.0, i as c_int);
            debug_assert!(!pointer.is_null());
            c_str_to_str!(pointer).unwrap()
        }
    }

    /// Return column names.
    #[inline]
    pub fn column_names(&self) -> Vec<&str> {
        (0..self.column_count())
            .map(|i| self.column_name(i))
            .collect()
    }

    /// Return the type of a column.
    ///
    /// The first column has index 0. The type becomes available after taking a step.
    pub fn column_type(&self, i: usize) -> Type {
        debug_assert!(i < self.column_count(), "the index is out of range");
        match unsafe { ffi::sqlite3_column_type(self.raw.0, i as c_int) } {
            ffi::SQLITE_BLOB => Type::Binary,
            ffi::SQLITE_FLOAT => Type::Float,
            ffi::SQLITE_INTEGER => Type::Integer,
            ffi::SQLITE_TEXT => Type::String,
            ffi::SQLITE_NULL => Type::Null,
            _ => unreachable!(),
        }
    }

    /// Advance to the next state.
    ///
    /// The function should be called multiple times until `State::Done` is
    /// reached in order to evaluate the statement entirely.
    pub fn next(&mut self) -> Result<State> {
        Ok(match unsafe { ffi::sqlite3_step(self.raw.0) } {
            ffi::SQLITE_ROW => State::Row,
            ffi::SQLITE_DONE => State::Done,
            code => error!(self.raw.1, code),
        })
    }

    /// Return the index for a named parameter if exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (name STRING)");
    /// let statement = connection.prepare("SELECT * FROM users WHERE name = :name")?;
    /// assert_eq!(statement.parameter_index(":name")?.unwrap(), 1);
    /// assert_eq!(statement.parameter_index(":asdf")?, None);
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    #[inline]
    pub fn parameter_index(&self, parameter: &str) -> Result<Option<usize>> {
        let index = unsafe {
            ffi::sqlite3_bind_parameter_index(self.raw.0, str_to_cstr!(parameter).as_ptr())
        };
        match index {
            0 => Ok(None),
            _ => Ok(Some(index as usize)),
        }
    }

    /// Read a value from a column.
    ///
    /// The first column has index 0.
    #[inline]
    pub fn read<T: Readable>(&self, i: usize) -> Result<T> {
        debug_assert!(i < self.column_count(), "the index is out of range");
        Readable::read(self, i)
    }

    /// Reset the statement.
    #[inline]
    pub fn reset(&mut self) -> Result<()> {
        unsafe { ok!(self.raw.1, ffi::sqlite3_reset(self.raw.0)) };
        Ok(())
    }

    /// Upgrade to a cursor.
    #[inline]
    pub fn into_cursor(self) -> Cursor<'l> {
        ::cursor::new(self)
    }

    /// Return the raw pointer.
    #[inline]
    pub fn as_raw(&self) -> *mut ffi::sqlite3_stmt {
        self.raw.0
    }

    #[deprecated(since = "0.26.0", note = "Please use `column_count` instead.")]
    pub fn count(&self) -> usize {
        self.column_count()
    }

    #[deprecated(since = "0.26.0", note = "Please use `into_cursor` instead.")]
    pub fn cursor(self) -> Cursor<'l> {
        self.into_cursor()
    }

    #[deprecated(since = "0.26.0", note = "Please use `column_name` instead.")]
    pub fn name(&self, i: usize) -> &str {
        self.column_name(i)
    }

    #[deprecated(since = "0.26.0", note = "Please use `column_names` instead.")]
    pub fn names(&self) -> Vec<&str> {
        self.column_names()
    }

    #[deprecated(since = "0.26.0", note = "Please use `column_type` instead.")]
    pub fn kind(&self, i: usize) -> Type {
        self.column_type(i)
    }
}

impl<'l> Drop for Statement<'l> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_finalize(self.raw.0) };
    }
}

impl<'l> Bindable for &'l Value {
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        match self {
            &Value::Binary(ref value) => (value as &[u8]).bind(statement, i),
            &Value::Float(value) => value.bind(statement, i),
            &Value::Integer(value) => value.bind(statement, i),
            &Value::String(ref value) => (value as &str).bind(statement, i),
            &Value::Null => ().bind(statement, i),
        }
    }
}

impl<'l> Bindable for &'l [u8] {
    #[inline]
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_blob(
                    statement.raw.0,
                    i as c_int,
                    self.as_ptr() as *const _,
                    self.len() as c_int,
                    transient!(),
                )
            );
        }
        Ok(())
    }
}

impl Bindable for f64 {
    #[inline]
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_double(statement.raw.0, i as c_int, self as c_double)
            );
        }
        Ok(())
    }
}

impl Bindable for i64 {
    #[inline]
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_int64(statement.raw.0, i as c_int, self as ffi::sqlite3_int64)
            );
        }
        Ok(())
    }
}

impl<'l> Bindable for &'l str {
    #[inline]
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_text(
                    statement.raw.0,
                    i as c_int,
                    self.as_ptr() as *const _,
                    self.len() as c_int,
                    transient!(),
                )
            );
        }
        Ok(())
    }
}

impl Bindable for () {
    #[inline]
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_null(statement.raw.0, i as c_int)
            );
        }
        Ok(())
    }
}

impl<T: Bindable> Bindable for Option<T> {
    #[inline]
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        match self {
            Some(inner) => inner.bind(statement, i),
            None => ().bind(statement, i),
        }
    }
}

impl Readable for Value {
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        Ok(match statement.column_type(i) {
            Type::Binary => Value::Binary(Readable::read(statement, i)?),
            Type::Float => Value::Float(Readable::read(statement, i)?),
            Type::Integer => Value::Integer(Readable::read(statement, i)?),
            Type::String => Value::String(Readable::read(statement, i)?),
            Type::Null => Value::Null,
        })
    }
}

impl Readable for f64 {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        Ok(unsafe { ffi::sqlite3_column_double(statement.raw.0, i as c_int) as f64 })
    }
}

impl Readable for i64 {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        Ok(unsafe { ffi::sqlite3_column_int64(statement.raw.0, i as c_int) as i64 })
    }
}

impl Readable for String {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        unsafe {
            let pointer = ffi::sqlite3_column_text(statement.raw.0, i as c_int);
            if pointer.is_null() {
                raise!("cannot read a text column");
            }
            Ok(c_str_to_string!(pointer))
        }
    }
}

impl Readable for Vec<u8> {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        use std::ptr::copy_nonoverlapping as copy;
        unsafe {
            let pointer = ffi::sqlite3_column_blob(statement.raw.0, i as c_int);
            if pointer.is_null() {
                return Ok(vec![]);
            }
            let count = ffi::sqlite3_column_bytes(statement.raw.0, i as c_int) as usize;
            let mut buffer = Vec::with_capacity(count);
            buffer.set_len(count);
            copy(pointer as *const u8, buffer.as_mut_ptr(), count);
            Ok(buffer)
        }
    }
}

impl<T: Readable> Readable for Option<T> {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        if statement.column_type(i) == Type::Null {
            Ok(None)
        } else {
            T::read(statement, i).map(Some)
        }
    }
}

#[inline]
pub fn new<'l, T: AsRef<str>>(raw1: *mut ffi::sqlite3, statement: T) -> Result<Statement<'l>> {
    let mut raw0 = 0 as *mut _;
    unsafe {
        ok!(
            raw1,
            ffi::sqlite3_prepare_v2(
                raw1,
                str_to_cstr!(statement.as_ref()).as_ptr(),
                -1,
                &mut raw0,
                0 as *mut _,
            )
        );
    }
    Ok(Statement {
        raw: (raw0, raw1),
        phantom: PhantomData,
    })
}
