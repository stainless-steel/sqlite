use libc::{c_double, c_int};
use raw;
use std::marker::PhantomData;

use {Result, ResultCode};

/// A prepared statement.
pub struct Statement<'l> {
    raw: *mut raw::sqlite3_stmt,
    _phantom: PhantomData<&'l raw::sqlite3_stmt>,
}

/// A binding of a prepared statement.
pub enum Binding<'l> {
    Float(usize, f64),
    Integer(usize, i64),
    Text(usize, &'l str),
}

impl<'l> Statement<'l> {
    /// Assign values to the placeholders.
    pub fn bind(&mut self, bindings: &[Binding]) -> Result<()> {
        for binding in bindings.iter() {
            match *binding {
                Binding::Float(i, value) => unsafe {
                    success!(raw::sqlite3_bind_double(self.raw, i as c_int, value as c_double));
                },
                Binding::Integer(i, value) => unsafe {
                    success!(raw::sqlite3_bind_int64(self.raw, i as c_int,
                                                     value as raw::sqlite3_int64));
                },
                Binding::Text(i, value) => unsafe {
                    success!(raw::sqlite3_bind_text(self.raw, i as c_int, str_to_c_str!(value),
                                                    -1, None));
                },
            }
        }
        Ok(())
    }

    /// Take a step.
    #[inline]
    pub fn step(&mut self) -> ResultCode {
        unsafe { ResultCode::from_raw(raw::sqlite3_step(self.raw)) }
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

#[inline]
pub fn from_raw<'l>(raw: *mut raw::sqlite3_stmt) -> Statement<'l> {
    Statement { raw: raw, _phantom: PhantomData }
}
