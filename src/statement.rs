use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

use libc::{c_double, c_int};

use crate::cursor::{Cursor, CursorWithOwnership, Row};
use crate::error::Result;
use crate::value::{Type, Value};

// https://sqlite.org/c3ref/c_static.html
macro_rules! transient(
    () => (std::mem::transmute(!0 as *const libc::c_void));
);

/// A prepared statement.
pub struct Statement<'l> {
    raw: (*mut ffi::sqlite3_stmt, *mut ffi::sqlite3),
    column_names: Vec<String>,
    column_mapping: Rc<HashMap<String, usize>>,
    phantom: PhantomData<(ffi::sqlite3_stmt, &'l ffi::sqlite3)>,
}

/// A type suitable for binding to a prepared statement.
pub trait Bindable {
    /// Bind to a parameter.
    fn bind(self, _: &mut Statement) -> Result<()>;
}

/// A type suitable for binding to a prepared statement given a parameter index.
pub trait BindableWithIndex {
    /// Bind to a parameter.
    ///
    /// In case of integer indices, the first parameter has index 1.
    fn bind<T: ParameterIndex>(self, _: &mut Statement, _: T) -> Result<()>;
}

/// A type suitable for indexing columns in a prepared statement.
pub trait ColumnIndex: Copy + std::fmt::Debug {
    /// Identify the ordinal position.
    ///
    /// The first column has index 0.
    fn index(self, statement: &Statement) -> Result<usize>;
}

/// A type suitable for indexing parameters in a prepared statement.
pub trait ParameterIndex: Copy + std::fmt::Debug {
    /// Identify the ordinal position.
    ///
    /// The first parameter has index 1.
    fn index(self, statement: &Statement) -> Result<usize>;
}

/// A type suitable for reading from a prepared statement given a column index.
pub trait ReadableWithIndex: Sized {
    /// Read from a column.
    ///
    /// In case of integer indices, the first column has index 0.
    fn read<T: ColumnIndex>(_: &Statement, _: T) -> Result<Self>;
}

/// The state of a prepared statement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// There is a row available for reading.
    Row,
    /// The statement has been entirely evaluated.
    Done,
}

impl<'l> Statement<'l> {
    /// Bind values to parameters.
    ///
    /// In case of integer indices, the first parameter has index 1.
    ///
    /// # Examples
    ///
    /// ```
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (id INTEGER, name STRING)");
    /// let query = "SELECT * FROM users WHERE name = ?";
    /// let mut statement = connection.prepare(query)?;
    /// statement.bind((1, "Bob"))?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    ///
    /// ```
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (id INTEGER, name STRING)");
    /// let query = "SELECT * FROM users WHERE name = ?";
    /// let mut statement = connection.prepare(query)?;
    /// statement.bind(&[(1, "Bob")][..])?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    ///
    /// ```
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (id INTEGER, name STRING)");
    /// let query = "SELECT * FROM users WHERE name = :name";
    /// let mut statement = connection.prepare(query)?;
    /// statement.bind((":name", "Bob"))?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    ///
    /// ```
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (id INTEGER, name STRING)");
    /// let query = "SELECT * FROM users WHERE name = :name";
    /// let mut statement = connection.prepare(query)?;
    /// statement.bind(&[(":name", "Bob")][..])?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    ///
    /// ```
    /// # use sqlite::Value;
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (id INTEGER, name STRING)");
    /// let query = "SELECT * FROM users WHERE id = :id AND name = :name";
    /// let mut statement = connection.prepare(query)?;
    /// statement.bind::<&[(_, Value)]>(&[
    ///     (":id", 1.into()),
    ///     (":name", "Bob".into()),
    /// ][..])?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    #[inline]
    pub fn bind<T: Bindable>(&mut self, value: T) -> Result<()> {
        value.bind(self)?;
        Ok(())
    }

    /// Bind values to parameters via an iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sqlite::Value;
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (id INTEGER, name STRING)");
    /// let query = "INSERT INTO users VALUES (:id, :name)";
    /// let mut statement = connection.prepare(query)?;
    /// statement.bind_iter::<_, (_, Value)>([
    ///     (":name", "Bob".into()),
    ///     (":id", 42.into()),
    /// ])?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    pub fn bind_iter<T, U>(&mut self, value: T) -> Result<()>
    where
        T: IntoIterator<Item = U>,
        U: Bindable,
    {
        for value in value {
            self.bind(value)?;
        }
        Ok(())
    }

    /// Create a cursor.
    #[inline]
    pub fn iter(&mut self) -> Cursor<'l, '_> {
        self.into()
    }

    /// Advance to the next state.
    ///
    /// The function should be called multiple times until `State::Done` is
    /// reached in order to evaluate the statement entirely.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<State> {
        Ok(match unsafe { ffi::sqlite3_step(self.raw.0) } {
            ffi::SQLITE_ROW => State::Row,
            ffi::SQLITE_DONE => State::Done,
            code => error!(self.raw.1, code),
        })
    }

    /// Read a value from a column.
    ///
    /// In case of integer indices, the first column has index 0.
    #[inline]
    pub fn read<T, U>(&self, index: U) -> Result<T>
    where
        T: ReadableWithIndex,
        U: ColumnIndex,
    {
        ReadableWithIndex::read(self, index)
    }

    /// Return the number of columns.
    #[inline]
    pub fn column_count(&self) -> usize {
        self.column_names.len()
    }

    #[doc(hidden)]
    #[inline]
    pub fn column_mapping(&self) -> Rc<HashMap<String, usize>> {
        self.column_mapping.clone()
    }

    /// Return the name of a column.
    ///
    /// In case of integer indices, the first column has index 0.
    #[inline]
    pub fn column_name<T: ColumnIndex>(&self, index: T) -> Result<&str> {
        Ok(&self.column_names[index.index(self)?])
    }

    /// Return column names.
    #[inline]
    pub fn column_names(&self) -> &[String] {
        &self.column_names
    }

    /// Return the type of a column.
    ///
    /// The type becomes available after taking a step. In case of integer
    /// indices, the first column has index 0.
    pub fn column_type<T: ColumnIndex>(&self, index: T) -> Result<Type> {
        Ok(
            match unsafe { ffi::sqlite3_column_type(self.raw.0, index.index(self)? as c_int) } {
                ffi::SQLITE_BLOB => Type::Binary,
                ffi::SQLITE_FLOAT => Type::Float,
                ffi::SQLITE_INTEGER => Type::Integer,
                ffi::SQLITE_TEXT => Type::String,
                ffi::SQLITE_NULL => Type::Null,
                _ => unreachable!(),
            },
        )
    }

    /// Return the index for a named parameter if exists.
    ///
    /// # Examples
    ///
    /// ```
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (name STRING)");
    /// let query = "SELECT * FROM users WHERE name = :name";
    /// let statement = connection.prepare(query)?;
    /// assert_eq!(statement.parameter_index(":name")?.unwrap(), 1);
    /// assert_eq!(statement.parameter_index(":asdf")?, None);
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    pub fn parameter_index(&self, parameter: &str) -> Result<Option<usize>> {
        let index = unsafe {
            ffi::sqlite3_bind_parameter_index(self.raw.0, str_to_cstr!(parameter).as_ptr())
        };
        match index {
            0 => Ok(None),
            _ => Ok(Some(index as usize)),
        }
    }

    /// Reset the internal state.
    #[inline]
    pub fn reset(&mut self) -> Result<()> {
        unsafe { ok!(self.raw.1, ffi::sqlite3_reset(self.raw.0)) };
        Ok(())
    }

    #[doc(hidden)]
    #[inline]
    pub fn as_raw(&self) -> *mut ffi::sqlite3_stmt {
        self.raw.0
    }
}

impl<'l> Drop for Statement<'l> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_finalize(self.raw.0) };
    }
}

impl<'l, 'm> From<&'m mut Statement<'l>> for Cursor<'l, 'm> {
    #[inline]
    fn from(statement: &'m mut Statement<'l>) -> Self {
        crate::cursor::new(statement)
    }
}

impl<'l> IntoIterator for Statement<'l> {
    type Item = Result<Row>;
    type IntoIter = CursorWithOwnership<'l>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        crate::cursor::new_with_ownership(self)
    }
}

impl<T, U> Bindable for (T, U)
where
    T: ParameterIndex,
    U: BindableWithIndex,
{
    #[inline]
    fn bind(self, statement: &mut Statement) -> Result<()> {
        self.1.bind(statement, self.0)
    }
}

impl<T> Bindable for &[T]
where
    T: BindableWithIndex + Clone,
{
    fn bind(self, statement: &mut Statement) -> Result<()> {
        for (index, value) in self.iter().enumerate() {
            value.clone().bind(statement, index + 1)?;
        }
        Ok(())
    }
}

impl<T, U> Bindable for &[(T, U)]
where
    T: ParameterIndex,
    U: BindableWithIndex + Clone,
{
    fn bind(self, statement: &mut Statement) -> Result<()> {
        for (index, value) in self.iter() {
            value.clone().bind(statement, *index)?;
        }
        Ok(())
    }
}

impl BindableWithIndex for &[u8] {
    fn bind<T: ParameterIndex>(self, statement: &mut Statement, index: T) -> Result<()> {
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_blob(
                    statement.raw.0,
                    index.index(statement)? as c_int,
                    self.as_ptr() as *const _,
                    self.len() as c_int,
                    transient!(),
                )
            );
        }
        Ok(())
    }
}

impl BindableWithIndex for f64 {
    fn bind<T: ParameterIndex>(self, statement: &mut Statement, index: T) -> Result<()> {
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_double(
                    statement.raw.0,
                    index.index(statement)? as c_int,
                    self as c_double
                )
            );
        }
        Ok(())
    }
}

impl BindableWithIndex for i64 {
    fn bind<T: ParameterIndex>(self, statement: &mut Statement, index: T) -> Result<()> {
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_int64(
                    statement.raw.0,
                    index.index(statement)? as c_int,
                    self as ffi::sqlite3_int64
                )
            );
        }
        Ok(())
    }
}

impl BindableWithIndex for &str {
    fn bind<T: ParameterIndex>(self, statement: &mut Statement, index: T) -> Result<()> {
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_text(
                    statement.raw.0,
                    index.index(statement)? as c_int,
                    self.as_ptr() as *const _,
                    self.len() as c_int,
                    transient!(),
                )
            );
        }
        Ok(())
    }
}

impl BindableWithIndex for () {
    fn bind<T: ParameterIndex>(self, statement: &mut Statement, index: T) -> Result<()> {
        unsafe {
            ok!(
                statement.raw.1,
                ffi::sqlite3_bind_null(statement.raw.0, index.index(statement)? as c_int)
            );
        }
        Ok(())
    }
}

impl BindableWithIndex for Value {
    #[inline]
    fn bind<T: ParameterIndex>(self, statement: &mut Statement, index: T) -> Result<()> {
        (index, &self).bind(statement)
    }
}

impl BindableWithIndex for &Value {
    fn bind<T: ParameterIndex>(self, statement: &mut Statement, index: T) -> Result<()> {
        match self {
            Value::Binary(ref value) => (value as &[u8]).bind(statement, index),
            Value::Float(value) => value.bind(statement, index),
            Value::Integer(value) => value.bind(statement, index),
            Value::String(ref value) => (value as &str).bind(statement, index),
            Value::Null => ().bind(statement, index),
        }
    }
}

impl<T> BindableWithIndex for Option<T>
where
    T: BindableWithIndex,
{
    #[inline]
    fn bind<U: ParameterIndex>(self, statement: &mut Statement, index: U) -> Result<()> {
        match self {
            Some(value) => value.bind(statement, index),
            None => ().bind(statement, index),
        }
    }
}

impl<T> BindableWithIndex for &Option<T>
where
    T: BindableWithIndex + Clone,
{
    #[inline]
    fn bind<U: ParameterIndex>(self, statement: &mut Statement, index: U) -> Result<()> {
        match self {
            Some(value) => value.clone().bind(statement, index),
            None => ().bind(statement, index),
        }
    }
}

impl ColumnIndex for &str {
    #[inline]
    fn index(self, statement: &Statement) -> Result<usize> {
        if statement.column_mapping.contains_key(self) {
            Ok(statement.column_mapping[self])
        } else {
            raise!("the index is out of range ({})", self);
        }
    }
}

impl ColumnIndex for usize {
    #[inline]
    fn index(self, statement: &Statement) -> Result<usize> {
        if self < statement.column_count() {
            Ok(self)
        } else {
            raise!("the index is out of range ({})", self);
        }
    }
}

impl ParameterIndex for &str {
    #[inline]
    fn index(self, statement: &Statement) -> Result<usize> {
        match statement.parameter_index(self)? {
            Some(index) => Ok(index),
            _ => raise!("the index is out of range ({})", self),
        }
    }
}

impl ParameterIndex for usize {
    #[inline]
    fn index(self, _: &Statement) -> Result<usize> {
        if self > 0 {
            Ok(self)
        } else {
            raise!("the index is out of range ({})", self);
        }
    }
}

impl ReadableWithIndex for Vec<u8> {
    fn read<T: ColumnIndex>(statement: &Statement, index: T) -> Result<Self> {
        use std::ptr::copy_nonoverlapping as copy;
        unsafe {
            let pointer =
                ffi::sqlite3_column_blob(statement.raw.0, index.index(statement)? as c_int);
            if pointer.is_null() {
                return Ok(vec![]);
            }
            let count = ffi::sqlite3_column_bytes(statement.raw.0, index.index(statement)? as c_int)
                as usize;
            let mut buffer = Vec::with_capacity(count);
            copy(pointer as *const u8, buffer.as_mut_ptr(), count);
            buffer.set_len(count);
            Ok(buffer)
        }
    }
}

impl ReadableWithIndex for f64 {
    #[allow(clippy::unnecessary_cast)]
    fn read<T: ColumnIndex>(statement: &Statement, index: T) -> Result<Self> {
        Ok(unsafe {
            ffi::sqlite3_column_double(statement.raw.0, index.index(statement)? as c_int) as f64
        })
    }
}

impl ReadableWithIndex for i64 {
    #[allow(clippy::unnecessary_cast)]
    fn read<T: ColumnIndex>(statement: &Statement, index: T) -> Result<Self> {
        Ok(unsafe {
            ffi::sqlite3_column_int64(statement.raw.0, index.index(statement)? as c_int) as i64
        })
    }
}

impl ReadableWithIndex for String {
    fn read<T: ColumnIndex>(statement: &Statement, index: T) -> Result<Self> {
        unsafe {
            let pointer =
                ffi::sqlite3_column_text(statement.raw.0, index.index(statement)? as c_int);
            if pointer.is_null() {
                raise!("cannot read a text column");
            }
            Ok(c_str_to_string!(pointer))
        }
    }
}

impl ReadableWithIndex for Value {
    fn read<T: ColumnIndex>(statement: &Statement, index: T) -> Result<Self> {
        Ok(match statement.column_type(index)? {
            Type::Binary => Value::Binary(ReadableWithIndex::read(statement, index)?),
            Type::Float => Value::Float(ReadableWithIndex::read(statement, index)?),
            Type::Integer => Value::Integer(ReadableWithIndex::read(statement, index)?),
            Type::String => Value::String(ReadableWithIndex::read(statement, index)?),
            Type::Null => Value::Null,
        })
    }
}

impl<T: ReadableWithIndex> ReadableWithIndex for Option<T> {
    fn read<U: ColumnIndex>(statement: &Statement, index: U) -> Result<Self> {
        if statement.column_type(index)? == Type::Null {
            Ok(None)
        } else {
            T::read(statement, index).map(Some)
        }
    }
}

pub fn new<'l, T>(raw_connection: *mut ffi::sqlite3, statement: T) -> Result<Statement<'l>>
where
    T: AsRef<str>,
{
    let mut raw_statement = std::ptr::null_mut();
    unsafe {
        ok!(
            raw_connection,
            ffi::sqlite3_prepare_v2(
                raw_connection,
                str_to_cstr!(statement.as_ref()).as_ptr(),
                -1,
                &mut raw_statement,
                std::ptr::null_mut(),
            )
        );
    }
    let column_count = unsafe { ffi::sqlite3_column_count(raw_statement) as usize };
    let column_names = (0..column_count)
        .map(|index| unsafe {
            let raw = ffi::sqlite3_column_name(raw_statement, index as c_int);
            debug_assert!(!raw.is_null());
            c_str_to_str!(raw).unwrap().to_string()
        })
        .collect::<Vec<_>>();
    let column_mapping = column_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.to_string(), index))
        .collect();
    Ok(Statement {
        raw: (raw_statement, raw_connection),
        column_names,
        column_mapping: Rc::new(column_mapping),
        phantom: PhantomData,
    })
}
