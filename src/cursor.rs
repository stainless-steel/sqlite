use ffi;
use statement::{State, Statement};
use {Result, Value};

/// An iterator over rows.
pub struct Cursor<'l> {
    state: Option<State>,
    values: Option<Vec<Value>>,
    statement: Statement<'l>,
}

impl<'l> Cursor<'l> {
    /// Bind values to all parameters.
    pub fn bind(&mut self, values: &[Value]) -> Result<()> {
        self.state = None;
        try!(self.statement.reset());
        for (i, value) in values.iter().enumerate() {
            try!(self.statement.bind(i + 1, value));
        }
        Ok(())
    }

    /// Return the number of columns.
    #[inline]
    pub fn columns(&self) -> usize {
        self.statement.columns()
    }

    /// Advance to the next row and read all columns.
    pub fn next(&mut self) -> Result<Option<&[Value]>> {
        match self.state {
            Some(State::Row) => {}
            Some(State::Done) => return Ok(None),
            _ => {
                self.state = Some(try!(self.statement.next()));
                return self.next();
            }
        }
        let values = match self.values.take() {
            Some(mut values) => {
                for (i, value) in values.iter_mut().enumerate() {
                    match value {
                        &mut Value::Binary(ref mut value) => {
                            *value = try!(self.statement.read(i));
                        }
                        &mut Value::Float(ref mut value) => {
                            *value = try!(self.statement.read(i));
                        }
                        &mut Value::Integer(ref mut value) => {
                            *value = try!(self.statement.read(i));
                        }
                        &mut Value::String(ref mut value) => {
                            *value = try!(self.statement.read(i));
                        }
                        &mut Value::Null => {}
                    }
                }
                values
            }
            _ => {
                let count = self.statement.columns();
                let mut values = Vec::with_capacity(count);
                for i in 0..count {
                    values.push(try!(self.statement.read(i)));
                }
                values
            }
        };
        self.state = Some(try!(self.statement.next()));
        self.values = Some(values);
        Ok(Some(self.values.as_ref().unwrap()))
    }

    /// Return the raw pointer.
    #[inline]
    pub fn as_raw(&self) -> *mut ffi::sqlite3_stmt {
        self.statement.as_raw()
    }
}

#[inline]
pub fn new<'l>(statement: Statement<'l>) -> Cursor<'l> {
    Cursor {
        state: None,
        values: None,
        statement: statement,
    }
}
