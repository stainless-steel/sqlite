use libc::{c_double, c_int};
use raw;
use std::marker::PhantomData;

use {Database, Result, ResultCode};

/// A prepared statement.
pub struct Statement<'l> {
    raw: *mut raw::sqlite3_stmt,
    phantom: PhantomData<(&'l raw::sqlite3, raw::sqlite3_stmt)>,
}

/// A binding of a prepared statement.
pub enum Binding<'l> {
    Float(usize, f64),
    Integer(usize, i64),
    Text(usize, &'l str),
}

/// A value stored in a prepared statement.
pub trait Value {
    /// Read the value at a specific column.
    fn read(statement: &mut Statement, i: usize) -> Result<Self>;
}

impl<'l> Statement<'l> {
    /// Assign values to the placeholders.
    ///
    /// The leftmost parameter has an index of 1.
    pub fn bind(&mut self, bindings: &[Binding]) -> Result<()> {
        for binding in bindings.iter() {
            match *binding {
                Binding::Float(i, value) => unsafe {
                    debug_assert!(i > 0, "the indexation starts from 1");
                    success!(raw::sqlite3_bind_double(self.raw, i as c_int, value as c_double));
                },
                Binding::Integer(i, value) => unsafe {
                    debug_assert!(i > 0, "the indexation starts from 1");
                    success!(raw::sqlite3_bind_int64(self.raw, i as c_int,
                                                     value as raw::sqlite3_int64));
                },
                Binding::Text(i, value) => unsafe {
                    debug_assert!(i > 0, "the indexation starts from 1");
                    success!(raw::sqlite3_bind_text(self.raw, i as c_int, str_to_c_str!(value),
                                                    -1, None));
                },
            }
        }
        Ok(())
    }

    /// Return the value of a column.
    #[inline]
    pub fn column<T: Value>(&mut self, i: usize) -> Result<T> {
        <T as Value>::read(self, i)
    }

    /// Take a step.
    #[inline]
    pub fn step(&mut self) -> ResultCode {
        unsafe { ::result::code_from_raw(raw::sqlite3_step(self.raw)) }
    }

    /// Reset.
    #[inline]
    pub fn reset(&mut self) -> Result<()> {
        unsafe { success!(raw::sqlite3_reset(self.raw)) };
        Ok(())
    }
}

impl<'l> Drop for Statement<'l> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ::raw::sqlite3_finalize(self.raw) };
    }
}

impl Value for f64 {
    fn read(statement: &mut Statement, i: usize) -> Result<f64> {
        Ok(unsafe { ::raw::sqlite3_column_double(statement.raw, i as c_int) as f64 })
    }
}

impl Value for i64 {
    fn read(statement: &mut Statement, i: usize) -> Result<i64> {
        Ok(unsafe { ::raw::sqlite3_column_int64(statement.raw, i as c_int) as i64 })
    }
}

impl Value for String {
    fn read(statement: &mut Statement, i: usize) -> Result<String> {
        unsafe {
            let pointer = ::raw::sqlite3_column_text(statement.raw, i as c_int);
            if pointer.is_null() {
                raise!("cannot read a TEXT column");
            }
            Ok(c_str_to_string!(pointer))
        }
    }
}

#[inline]
pub fn new<'l>(database: &'l Database, sql: &str) -> Result<Statement<'l>> {
    let mut raw = 0 as *mut _;
    unsafe {
        success!(database, raw::sqlite3_prepare(::database::as_raw(database), str_to_c_str!(sql),
                                                -1, &mut raw, 0 as *mut _));
    }
    Ok(Statement { raw: raw, phantom: PhantomData })
}
