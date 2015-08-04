use ffi;
use libc::{c_double, c_int};
use std::marker::PhantomData;

use {Cursor, Result, Type, Value};

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
    /// The leftmost parameter has the index 1.
    fn bind(&self, &mut Statement, usize) -> Result<()>;
}

/// A type suitable for reading from a prepared statement.
pub trait Readable {
    /// Read from a column.
    ///
    /// The leftmost column has the index 0.
    fn read(&Statement, usize) -> Result<Self>;
}

impl<'l> Statement<'l> {
    /// Bind a value to a parameter.
    ///
    /// The leftmost parameter has the index 1.
    #[inline]
    pub fn bind<T: Bindable>(&mut self, i: usize, parameter: T) -> Result<()> {
        parameter.bind(self, i)
    }

    /// Return the number of columns.
    #[inline]
    pub fn columns(&self) -> usize {
        unsafe { ffi::sqlite3_column_count(self.raw.0) as usize }
    }

    /// Advance to the next state.
    ///
    /// The function should be called multiple times until `State::Done` is
    /// reached in order to evaluate the statement entirely.
    pub fn next(&mut self) -> Result<State> {
        let state = match unsafe { ffi::sqlite3_step(self.raw.0) } {
            ffi::SQLITE_ROW => State::Row,
            ffi::SQLITE_DONE => State::Done,
            code => error!(self.raw.1, code),
        };
        Ok(state)
    }

    /// Return the type of a column.
    ///
    /// The type is revealed after the first step has been taken.
    pub fn kind(&self, i: usize) -> Type {
        match unsafe { ffi::sqlite3_column_type(self.raw.0, i as c_int) } {
            ffi::SQLITE_BLOB => Type::Binary,
            ffi::SQLITE_FLOAT => Type::Float,
            ffi::SQLITE_INTEGER => Type::Integer,
            ffi::SQLITE_TEXT => Type::String,
            ffi::SQLITE_NULL => Type::Null,
            _ => unreachable!(),
        }
    }

    /// Read a value from a column.
    ///
    /// The leftmost column has the index 0.
    #[inline]
    pub fn read<T: Readable>(&self, i: usize) -> Result<T> {
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
    pub fn cursor(self) -> Cursor<'l> {
        ::cursor::new(self)
    }
}

impl<'l> Drop for Statement<'l> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_finalize(self.raw.0) };
    }
}

impl Bindable for Value {
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
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
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(statement.raw.1, ffi::sqlite3_bind_blob(statement.raw.0, i as c_int,
                                                        self.as_ptr() as *const _,
                                                        self.len() as c_int, None));
        }
        Ok(())
    }
}

impl Bindable for f64 {
    #[inline]
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(statement.raw.1, ffi::sqlite3_bind_double(statement.raw.0, i as c_int,
                                                          *self as c_double));
        }
        Ok(())
    }
}

impl Bindable for i64 {
    #[inline]
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(statement.raw.1, ffi::sqlite3_bind_int64(statement.raw.0, i as c_int,
                                                         *self as ffi::sqlite3_int64));
        }
        Ok(())
    }
}

impl<'l> Bindable for &'l str {
    #[inline]
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(statement.raw.1, ffi::sqlite3_bind_text(statement.raw.0, i as c_int,
                                                        str_to_cstr!(*self).as_ptr(), -1, None));
        }
        Ok(())
    }
}

impl Bindable for () {
    #[inline]
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            ok!(statement.raw.1, ffi::sqlite3_bind_null(statement.raw.0, i as c_int));
        }
        Ok(())
    }
}

impl<'l, T: Bindable> Bindable for &'l T {
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        (*self).bind(statement, i)
    }
}

impl Readable for Value {
    fn read(statement: &Statement, i: usize) -> Result<Self> {
        Ok(match statement.kind(i) {
            Type::Binary => Value::Binary(try!(Readable::read(statement, i))),
            Type::Float => Value::Float(try!(Readable::read(statement, i))),
            Type::Integer => Value::Integer(try!(Readable::read(statement, i))),
            Type::String => Value::String(try!(Readable::read(statement, i))),
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

#[inline]
pub fn new<'l, T: AsRef<str>>(raw1: *mut ffi::sqlite3, statement: T) -> Result<Statement<'l>> {
    let mut raw0 = 0 as *mut _;
    unsafe {
        ok!(raw1, ffi::sqlite3_prepare_v2(raw1, str_to_cstr!(statement.as_ref()).as_ptr(), -1,
                                          &mut raw0, 0 as *mut _));
    }
    Ok(Statement { raw: (raw0, raw1), phantom: PhantomData })
}
