use libc::{c_char, c_int, c_void};
use raw;
use std::marker::PhantomData;
use std::path::Path;

use {Result, Statement};

/// A connection to a database.
pub struct Database<'l> {
    raw: *mut raw::sqlite3,
    busy_callback: Option<Box<FnMut(usize) -> bool + 'l>>,
    phantom: PhantomData<raw::sqlite3>,
}

impl<'l> Database<'l> {
    /// Open a connection to a new or existing database.
    pub fn open(path: &Path) -> Result<Database<'l>> {
        let mut raw = 0 as *mut _;
        unsafe {
            success!(raw::sqlite3_open(path_to_c_str!(path), &mut raw));
        }
        Ok(Database {
            raw: raw,
            busy_callback: None,
            phantom: PhantomData,
        })
    }

    /// Execute a query without processing the resulting rows if any.
    #[inline]
    pub fn execute(&self, sql: &str) -> Result<()> {
        unsafe {
            success!(self.raw, raw::sqlite3_exec(self.raw, str_to_c_str!(sql), None, 0 as *mut _,
                                                 0 as *mut _));
        }
        Ok(())
    }

    /// Execute a query and process the resulting rows if any.
    ///
    /// The callback is triggered for each row. If the callback returns `false`,
    /// no more rows will be processed.
    #[inline]
    pub fn process<F>(&self, sql: &str, callback: F) -> Result<()>
        where F: FnMut(&[(&str, Option<&str>)]) -> bool
    {
        unsafe {
            let callback = Box::new(callback);
            success!(self.raw, raw::sqlite3_exec(self.raw, str_to_c_str!(sql),
                                                 Some(process_callback::<F>),
                                                 &*callback as *const F as *mut F as *mut _,
                                                 0 as *mut _));
        }
        Ok(())
    }

    /// Create a prepared statement.
    #[inline]
    pub fn prepare(&'l self, sql: &str) -> Result<Statement<'l>> {
        ::statement::new(self.raw, sql)
    }

    /// Set a callback for handling busy events.
    ///
    /// The callback is triggered when the database cannot perform an operation
    /// due to processing of some other request. If the callback returns `true`,
    /// the operation will be repeated.
    pub fn set_busy_handler<F>(&mut self, callback: F) -> Result<()>
        where F: FnMut(usize) -> bool + 'l
    {
        try!(self.remove_busy_handler());
        unsafe {
            let callback = Box::new(callback);
            let result = raw::sqlite3_busy_handler(self.raw, Some(busy_callback::<F>),
                                                   &*callback as *const F as *mut F as *mut _);
            self.busy_callback = Some(callback);
            success!(self.raw, result);
        }
        Ok(())
    }

    /// Set an implicit callback for handling busy events that tries to repeat
    /// rejected operations until a timeout expires.
    #[inline]
    pub fn set_busy_timeout(&mut self, milliseconds: usize) -> Result<()> {
        unsafe { success!(self.raw, raw::sqlite3_busy_timeout(self.raw, milliseconds as c_int)) };
        Ok(())
    }

    /// Remove the callback handling busy events.
    #[inline]
    pub fn remove_busy_handler(&mut self) -> Result<()> {
        ::std::mem::replace(&mut self.busy_callback, None);
        unsafe { success!(self.raw, raw::sqlite3_busy_handler(self.raw, None, 0 as *mut _)) };
        Ok(())
    }
}

impl<'l> Drop for Database<'l> {
    #[inline]
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.remove_busy_handler();
        unsafe { raw::sqlite3_close(self.raw) };
    }
}

extern fn busy_callback<F>(callback: *mut c_void, attempts: c_int) -> c_int
    where F: FnMut(usize) -> bool
{
    unsafe { if (*(callback as *mut F))(attempts as usize) { 1 } else { 0 } }
}

extern fn process_callback<F>(callback: *mut c_void, count: c_int, values: *mut *mut c_char,
                              columns: *mut *mut c_char) -> c_int
    where F: FnMut(&[(&str, Option<&str>)]) -> bool
{
    use std::str;
    use std::ffi::CStr;

    macro_rules! c_str_to_str(
        ($string:expr) => (str::from_utf8(CStr::from_ptr($string).to_bytes()));
    );

    unsafe {
        let mut pairs = Vec::with_capacity(count as usize);

        for i in 0..(count as isize) {
            let column = {
                let pointer = *columns.offset(i);
                debug_assert!(!pointer.is_null());
                c_str_to_str!(pointer).unwrap()
            };
            let value = {
                let pointer = *values.offset(i);
                if pointer.is_null() {
                    None
                } else {
                    Some(c_str_to_str!(pointer).unwrap())
                }
            };
            pairs.push((column, value));
        }

        if (*(callback as *mut F))(&pairs) { 0 } else { 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::Database;
    use tests::setup;

    macro_rules! ok(
        ($result:expr) => ($result.unwrap());
    );

    #[test]
    fn execute() {
        let (path, _directory) = setup();
        let database = ok!(Database::open(&path));
        match database.execute(":)") {
            Err(error) => assert_eq!(error.message,
                                     Some(String::from(r#"unrecognized token: ":""#))),
            _ => assert!(false),
        }
    }

    #[test]
    fn set_busy_handler() {
        let (path, _directory) = setup();
        let mut database = ok!(Database::open(&path));
        ok!(database.set_busy_handler(|_| true));
    }
}
