use crate::sqlite3_connector as ffi;
use crate::{Cursor, Result, Type, Value};

use std::marker::PhantomData;

/// A prepared statement.
pub struct Statement {
    raw: (ffi::Sqlite3StmtHandle, ffi::Sqlite3DbHandle),
    phantom: PhantomData<(ffi::Sqlite3StmtHandle, ffi::Sqlite3DbHandle)>,
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
    /// The leftmost parameter has the index 1.
    fn bind(self, _: &mut Statement, _: usize) -> Result<()>;
}

/// A type suitable for reading from a prepared statement.
pub trait Readable: Sized {
    /// Read from a column.
    ///
    /// The leftmost column has the index 0.
    fn read(_: &Statement, _: usize) -> Result<Self>;
}

impl Statement {
    /// Bind a value to a parameter.
    ///
    /// The leftmost parameter has the index 1.
    #[inline]
    pub fn bind<T: Bindable>(&mut self, i: usize, value: T) -> Result<()> {
        value.bind(self, i)
    }

    /// Return the number of columns.
    #[inline]
    pub fn count(&self) -> usize {
        unsafe { ffi::sqlite3_column_count(self.raw.0) as usize }
    }

    /// Return the type of a column.
    ///
    /// The type becomes available after taking a step.
    pub fn kind(&self, i: usize) -> Type {
        debug_assert!(i < self.count(), "the index is out of range");
        match unsafe { ffi::sqlite3_column_type(self.raw.0, i as _) } {
            ffi::SQLITE_BLOB => Type::Binary,
            ffi::SQLITE_FLOAT => Type::Float,
            ffi::SQLITE_INTEGER => Type::Integer,
            ffi::SQLITE_TEXT => Type::String,
            ffi::SQLITE_NULL => Type::Null,
            _ => unreachable!(),
        }
    }

    /// Return the name of a column.
    #[inline]
    pub fn name(&self, i: usize) -> String {
        debug_assert!(i < self.count(), "the index is out of range");
        unsafe { ffi::sqlite3_column_name(self.raw.0, i as _) }
    }

    /// Return column names.
    #[inline]
    pub fn names(&self) -> Vec<String> {
        (0..self.count()).map(|i| self.name(i)).collect()
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

    /// Read a value from a column.
    ///
    /// The leftmost column has the index 0.
    #[inline]
    pub fn read<T: Readable>(&self, i: usize) -> Result<T> {
        debug_assert!(i < self.count(), "the index is out of range");
        Readable::read(self, i)
    }

    /// Reset the statement.
    #[inline]
    pub fn reset(&mut self) -> Result<()> {
        unsafe { ok_raw!(self.raw.1, ffi::sqlite3_reset(self.raw.0)) };
        Ok(())
    }

    /// Upgrade to a cursor.
    #[inline]
    pub fn cursor(self) -> Cursor {
        crate::cursor::new(self)
    }

    /// Return the raw pointer.
    #[inline]
    pub fn as_raw(&self) -> ffi::Sqlite3StmtHandle {
        self.raw.0
    }
}

impl<'l> Drop for Statement {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_finalize(self.raw.0) };
    }
}

impl Bindable for &Value {
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        match self {
            Value::Binary(value) => value.bind(statement, i),
            &Value::Float(value) => value.bind(statement, i),
            &Value::Integer(value) => value.bind(statement, i),
            Value::String(value) => value.as_str().bind(statement, i),
            Value::Null => ().bind(statement, i),
        }
    }
}

impl Bindable for &Vec<u8> {
    #[inline]
    fn bind(self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok_raw!(
                statement.raw.1,
                ffi::sqlite3_bind_blob(statement.raw.0, i as _, self.into(), 0)
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
            ok_raw!(
                statement.raw.1,
                ffi::sqlite3_bind_double(statement.raw.0, i as i32, self)
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
            ok_raw!(
                statement.raw.1,
                ffi::sqlite3_bind_int64(statement.raw.0, i as i32, self as _)
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
            ok_raw!(
                statement.raw.1,
                ffi::sqlite3_bind_text(statement.raw.0, i as i32, self.into(), 0)
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
            ok_raw!(
                statement.raw.1,
                ffi::sqlite3_bind_null(statement.raw.0, i as i32)
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
        Ok(match statement.kind(i) {
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
        Ok(unsafe { ffi::sqlite3_column_double(statement.raw.0, i as _) })
    }
}

impl Readable for i64 {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        Ok(unsafe { ffi::sqlite3_column_int64(statement.raw.0, i as _) })
    }
}

impl Readable for String {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        unsafe { Ok(ffi::sqlite3_column_text(statement.raw.0, i as _)) }
    }
}

impl Readable for Vec<u8> {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        unsafe { Ok(ffi::sqlite3_column_blob(statement.raw.0, i as i32)) }
    }
}

impl<T: Readable> Readable for Option<T> {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        if statement.kind(i) == Type::Null {
            Ok(None)
        } else {
            T::read(statement, i).map(Some)
        }
    }
}

#[inline]
pub fn new<T: AsRef<str>>(raw1: ffi::Sqlite3DbHandle, statement: T) -> Result<Statement> {
    let result = unsafe { ffi::sqlite3_prepare_v2(raw1, statement.as_ref().into()) };
    if result.ret_code != ffi::SQLITE_OK {
        error!(raw1, result.ret_code)
    }

    Ok(Statement {
        raw: (result.stmt_handle, raw1),
        phantom: PhantomData,
    })
}
