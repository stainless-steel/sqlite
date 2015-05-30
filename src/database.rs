use libc::{c_char, c_int, c_void};
use raw;
use std::marker::PhantomData;
use std::path::Path;

use {Result, Statement};

/// A database.
pub struct Database<'l> {
    raw: *mut raw::sqlite3,
    phantom: PhantomData<&'l raw::sqlite3>,
}

/// A callback triggered for each row of an executed SQL query.
pub type ExecuteCallback<'l> = FnMut(Vec<(String, String)>) -> bool + 'l;

impl<'l> Database<'l> {
    /// Open a database.
    pub fn open(path: &Path) -> Result<Database> {
        let mut raw = 0 as *mut _;
        unsafe {
            success!(raw::sqlite3_open(path_to_c_str!(path), &mut raw));
        }
        Ok(Database { raw: raw, phantom: PhantomData })
    }

    /// Execute an SQL statement.
    pub fn execute<'c>(&mut self, sql: &str,
                       callback: Option<&mut ExecuteCallback<'c>>) -> Result<()> {

        unsafe {
            match callback {
                Some(callback) => {
                    let mut callback = Box::new(callback);
                    success!(self, raw::sqlite3_exec(self.raw, str_to_c_str!(sql),
                                                     Some(execute_callback),
                                                     &mut callback as *mut _ as *mut _,
                                                     0 as *mut _));
                },
                None => {
                    success!(self, raw::sqlite3_exec(self.raw, str_to_c_str!(sql), None,
                                                     0 as *mut _, 0 as *mut _));
                },
            }
        }

        Ok(())
    }

    /// Create a prepared statement.
    #[inline]
    pub fn statement(&mut self, sql: &str) -> Result<Statement<'l>> {
        ::statement::new(self, sql)
    }
}

impl<'l> Drop for Database<'l> {
    #[inline]
    fn drop(&mut self) {
        unsafe { raw::sqlite3_close(self.raw) };
    }
}

#[inline]
pub fn as_raw(database: &mut Database) -> *mut raw::sqlite3 {
    database.raw
}

extern fn execute_callback(callback: *mut c_void, count: c_int, values: *mut *mut c_char,
                           columns: *mut *mut c_char) -> c_int {

    unsafe {
        let mut pairs = Vec::with_capacity(count as usize);

        for i in 0..(count as isize) {
            let column = c_str_to_string!(*columns.offset(i));
            let value = c_str_to_string!(*values.offset(i));
            pairs.push((column, value));
        }

        let ref mut callback = *(callback as *mut Box<&mut ExecuteCallback>);
        if callback(pairs) { 0 } else { 1 }
    }
}
