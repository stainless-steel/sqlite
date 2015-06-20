use ffi;
use libc::{c_double, c_int};
use std::marker::PhantomData;

use Result;

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
    /// There is nothing else to read.
    Done,
}

/// A parameter of a prepared statement.
pub trait Parameter {
    /// Bind the parameter at a specific location.
    ///
    /// The leftmost location has the index 1.
    fn bind(&self, &mut Statement, usize) -> Result<()>;
}

/// A value stored in a prepared statement.
pub trait Value {
    /// Read the value stored in a specific column.
    ///
    /// The leftmost column has the index 0.
    fn read(&Statement, usize) -> Result<Self>;
}

impl<'l> Statement<'l> {
    /// Bind the parameter at a specific location.
    ///
    /// The leftmost location has the index 1.
    #[inline]
    pub fn bind<T: Parameter>(&mut self, i: usize, parameter: T) -> Result<()> {
        parameter.bind(self, i)
    }

    /// Read the value stored in a specific column.
    ///
    /// The leftmost column has the index 0.
    #[inline]
    pub fn read<T: Value>(&self, i: usize) -> Result<T> {
        Value::read(self, i)
    }

    /// Evaluate the statement.
    pub fn step(&mut self) -> Result<State> {
        match unsafe { ffi::sqlite3_step(self.raw.0) } {
            ffi::SQLITE_DONE => Ok(State::Done),
            ffi::SQLITE_ROW => Ok(State::Row),
            code => failure!(self.raw.1, code),
        }
    }

    /// Reset the statement.
    #[inline]
    pub fn reset(&mut self) -> Result<()> {
        unsafe { success!(self.raw.1, ffi::sqlite3_reset(self.raw.0)) };
        Ok(())
    }
}

impl<'l> Drop for Statement<'l> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_finalize(self.raw.0) };
    }
}

impl Parameter for f64 {
    #[inline]
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            success!(statement.raw.1, ffi::sqlite3_bind_double(statement.raw.0, i as c_int,
                                                               *self as c_double));
        }
        Ok(())
    }
}

impl Parameter for i64 {
    #[inline]
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            success!(statement.raw.1, ffi::sqlite3_bind_int64(statement.raw.0, i as c_int,
                                                              *self as ffi::sqlite3_int64));
        }
        Ok(())
    }
}

impl<'l> Parameter for &'l str {
    #[inline]
    fn bind(&self, statement: &mut Statement, i: usize) -> Result<()> {
        debug_assert!(i > 0, "the indexing starts from 1");
        unsafe {
            success!(statement.raw.1, ffi::sqlite3_bind_text(statement.raw.0, i as c_int,
                                                             str_to_c_str!(*self), -1, None));
        }
        Ok(())
    }
}

impl Value for f64 {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<f64> {
        Ok(unsafe { ffi::sqlite3_column_double(statement.raw.0, i as c_int) as f64 })
    }
}

impl Value for i64 {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<i64> {
        Ok(unsafe { ffi::sqlite3_column_int64(statement.raw.0, i as c_int) as i64 })
    }
}

impl Value for String {
    #[inline]
    fn read(statement: &Statement, i: usize) -> Result<String> {
        unsafe {
            let pointer = ffi::sqlite3_column_text(statement.raw.0, i as c_int);
            if pointer.is_null() {
                raise!("cannot read a text column");
            }
            Ok(c_str_to_string!(pointer))
        }
    }
}

#[inline]
pub fn new<'l>(raw1: *mut ffi::sqlite3, sql: &str) -> Result<Statement<'l>> {
    let mut raw0 = 0 as *mut _;
    unsafe {
        success!(raw1, ffi::sqlite3_prepare_v2(raw1, str_to_c_str!(sql), -1, &mut raw0,
                                               0 as *mut _));
    }
    Ok(Statement { raw: (raw0, raw1), phantom: PhantomData })
}
