use libc::{c_double, c_int};
use raw;
use std::marker::PhantomData;

use Result;

/// A prepared statement.
pub struct Statement<'l> {
    raw: (*mut raw::sqlite3_stmt, *mut raw::sqlite3),
    phantom: PhantomData<(raw::sqlite3_stmt, &'l raw::sqlite3)>,
}

/// A binding of a parameter of a prepared statement.
pub enum Binding<'l> {
    Float(usize, f64),
    Integer(usize, i64),
    Text(usize, &'l str),
}

/// A value stored in a result row of a query.
pub trait Value {
    /// Read the value stored in a specific column.
    fn read(statement: &Statement, i: usize) -> Result<Self>;
}

/// A state of a prepared statement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Done,
    Row,
}

impl<'l> Statement<'l> {
    /// Bind values to the parameters.
    ///
    /// The leftmost parameter has the index 1.
    pub fn bind(&mut self, bindings: &[Binding]) -> Result<()> {
        for binding in bindings.iter() {
            match *binding {
                Binding::Float(i, value) => unsafe {
                    debug_assert!(i > 0, "the indexing starts from 1");
                    success!(self.raw.1, raw::sqlite3_bind_double(self.raw.0, i as c_int,
                                                                  value as c_double));
                },
                Binding::Integer(i, value) => unsafe {
                    debug_assert!(i > 0, "the indexing starts from 1");
                    success!(self.raw.1, raw::sqlite3_bind_int64(self.raw.0, i as c_int,
                                                                 value as raw::sqlite3_int64));
                },
                Binding::Text(i, value) => unsafe {
                    debug_assert!(i > 0, "the indexing starts from 1");
                    success!(self.raw.1, raw::sqlite3_bind_text(self.raw.0, i as c_int,
                                                                str_to_c_str!(value), -1, None));
                },
            }
        }
        Ok(())
    }

    /// Return the value stored in a specific column of the current result row.
    ///
    /// The leftmost column has the index 0.
    #[inline]
    pub fn column<T: Value>(&self, i: usize) -> Result<T> {
        <T as Value>::read(self, i)
    }

    /// Evaluate the statement.
    pub fn step(&mut self) -> Result<State> {
        match unsafe { raw::sqlite3_step(self.raw.0) } {
            raw::SQLITE_DONE => Ok(State::Done),
            raw::SQLITE_ROW => Ok(State::Row),
            code => failure!(self.raw.1, code),
        }
    }

    /// Reset the statement.
    #[inline]
    pub fn reset(&mut self) -> Result<()> {
        unsafe { success!(self.raw.1, raw::sqlite3_reset(self.raw.0)) };
        Ok(())
    }
}

impl<'l> Drop for Statement<'l> {
    #[inline]
    fn drop(&mut self) {
        unsafe { raw::sqlite3_finalize(self.raw.0) };
    }
}

impl Value for f64 {
    fn read(statement: &Statement, i: usize) -> Result<f64> {
        Ok(unsafe { raw::sqlite3_column_double(statement.raw.0, i as c_int) as f64 })
    }
}

impl Value for i64 {
    fn read(statement: &Statement, i: usize) -> Result<i64> {
        Ok(unsafe { raw::sqlite3_column_int64(statement.raw.0, i as c_int) as i64 })
    }
}

impl Value for String {
    fn read(statement: &Statement, i: usize) -> Result<String> {
        unsafe {
            let pointer = raw::sqlite3_column_text(statement.raw.0, i as c_int);
            if pointer.is_null() {
                raise!("cannot read a TEXT column");
            }
            Ok(c_str_to_string!(pointer))
        }
    }
}

#[inline]
pub fn new<'l>(raw1: *mut raw::sqlite3, sql: &str) -> Result<Statement<'l>> {
    let mut raw0 = 0 as *mut _;
    unsafe {
        success!(raw1, raw::sqlite3_prepare(raw1, str_to_c_str!(sql), -1, &mut raw0, 0 as *mut _));
    }
    Ok(Statement { raw: (raw0, raw1), phantom: PhantomData })
}
