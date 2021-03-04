
use std::marker::PhantomData;

use Result;

fn execute(raw: *mut ffi::sqlite3, statement: &str) -> Result<()> {
    unsafe {
        ok!(
            raw,
            ffi::sqlite3_exec(
                raw,
                str_to_cstr!(statement).as_ptr(),
                None,
                0 as *mut _,
                0 as *mut _,
            )
        );
    }
    Ok(())
}

/// A transaction scope
pub struct Transaction<'l> {
    raw: Option<*mut ffi::sqlite3>,
    phantom: PhantomData<&'l ffi::sqlite3>,
}

impl<'l> Transaction<'l> {
    /// Commit the transaction.
    #[inline]
    pub fn commit(&mut self) -> Result<()> {
        if let Some(raw) = self.raw.take() {
            return execute(raw, &"COMMIT");
        } else {
            return Err(::Error { code: None, message: Some(String::from("Transaction already consumed")) });
        }
    }

    /// Roll back the transaction.
    #[inline]
    pub fn rollback(&mut self) -> Result<()> {
        if let Some(raw) = self.raw.take() {
            return execute(raw, &"ROLLBACK");
        } else {
            return Err(::Error { code: None, message: Some(String::from("Transaction already consumed")) });
        }
    }
}

impl<'l> Drop for Transaction<'l> {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.rollback();
    }
}

#[inline]
pub fn new<'l>(raw: *mut ffi::sqlite3) -> Result<Transaction<'l>> {
    execute(raw, &"BEGIN")?;
    Ok(Transaction { raw: Some(raw), phantom: PhantomData })
}


/// A savepoint scope
pub struct Savepoint<'l> {
    raw: Option<*mut ffi::sqlite3>,
    name: String,
    phantom: PhantomData<&'l ffi::sqlite3>,
}

impl<'l> Savepoint<'l> {
    /// Release the savepoint.
    #[inline]
    pub fn release(&mut self) -> Result<()> {
        if let Some(raw) = self.raw.take() {
            return execute(raw, &format!("RELEASE {}", self.name));
        } else {
            return Err(::Error { code: None, message: Some(format!("Savepoint {} already consumed", self.name)) });
        }
    }

    /// Roll back to the savepoint.
    #[inline]
    pub fn rollback(&mut self) -> Result<()> {
        if let Some(raw) = self.raw.take() {
            return execute(raw, &format!("ROLLBACK TO {}", self.name));
        } else {
            return Err(::Error { code: None, message: Some(format!("Savepoint {} already consumed", self.name)) });
        }
    }
}

impl<'l> Drop for Savepoint<'l> {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.rollback();
    }
}

#[inline]
pub fn new_savepoint<'l>(raw: *mut ffi::sqlite3, name: &str) -> Result<Savepoint<'l>> {
    execute(raw, &format!("SAVEPOINT {}", name))?;
    Ok(Savepoint { raw: Some(raw), name: name.to_owned(), phantom: PhantomData })
}
