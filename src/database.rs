use libc::{c_char, c_int, c_void};
use raw;
use std::marker::PhantomData;
use std::path::Path;

use Result;

/// A database.
pub struct Database<'d> {
    db: *mut raw::sqlite3,
    _phantom: PhantomData<&'d raw::sqlite3>,
}

/// A callback executed for each row of the result of an SQL query.
pub type ExecuteCallback<'c> = FnMut(Vec<(String, String)>) -> bool + 'c;

impl<'d> Database<'d> {
    /// Open a database.
    pub fn open(path: &Path) -> Result<Database<'d>> {
        let mut db = 0 as *mut _;
        unsafe {
            success!(raw::sqlite3_open(path_to_c_str!(path), &mut db));
        }
        Ok(Database { db: db, _phantom: PhantomData })
    }

    /// Execute an SQL statement.
    pub fn execute<'c>(&mut self, sql: &str,
                       callback: Option<&mut ExecuteCallback<'c>>) -> Result<()> {

        unsafe {
            match callback {
                Some(callback) => {
                    let mut callback = Box::new(callback);
                    success!(raw::sqlite3_exec(self.db, str_to_c_str!(sql), Some(execute_callback),
                                               &mut callback as *mut _ as *mut _, 0 as *mut _));
                },
                None => {
                    success!(raw::sqlite3_exec(self.db, str_to_c_str!(sql), None,
                                               0 as *mut _, 0 as *mut _));
                },
            }
        }

        Ok(())
    }
}

impl<'d> Drop for Database<'d> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ::raw::sqlite3_close(self.db) };
    }
}

extern fn execute_callback(callback: *mut c_void, count: c_int, values: *mut *mut c_char,
                           columns: *mut *mut c_char) -> c_int {

    macro_rules! c_str_to_string(
        ($string:expr) => (
            match ::std::str::from_utf8(::std::ffi::CStr::from_ptr($string).to_bytes()) {
                Ok(string) => String::from(string),
                Err(_) => return 1,
            }
        );
    );

    unsafe {
        let mut pairs = Vec::with_capacity(count as usize);

        for i in 0..(count as isize) {
            let column = c_str_to_string!(*columns.offset(i) as *const _);
            let value = c_str_to_string!(*values.offset(i) as *const _);
            pairs.push((column, value));
        }

        let ref mut callback = *(callback as *mut Box<&mut ExecuteCallback>);
        if callback(pairs) { 0 } else { 1 }
    }
}
