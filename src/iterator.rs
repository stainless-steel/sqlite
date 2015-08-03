use statement::{State, Statement, Bindable, Readable};
use {Result, Value};

/// A reusable iterator over the results of a prepared statement.
pub struct Iterator<'l> {
    state: Option<State>,
    values: Option<Vec<Value>>,
    statement: Statement<'l>,
}

impl<'l> Iterator<'l> {
    /// Bind parameters and start iterating over the resulting rows.
    ///
    /// The function assigns values to the parameters of the underlaying
    /// prepared statement, execute it, and start iterating over the resulting
    /// rows.
    pub fn start(&mut self, values: &[Value]) -> Result<()> {
        try!(self.statement.reset());
        for (i, value) in values.iter().enumerate() {
            try!(self.statement.bind(i + 1, value));
        }
        self.state = Some(try!(self.statement.step()));
        Ok(())
    }

    /// Read the next row.
    pub fn next(&mut self) -> Result<Option<&[Value]>> {
        match self.state {
            Some(State::Row) => {},
            _ => return Ok(None),
        }
        let values = match self.values.take() {
            Some(mut values) => {
                for (i, value) in values.iter_mut().enumerate() {
                    *value = try!(self.statement.read(i));
                }
                values
            },
            _ => {
                let count = self.statement.columns();
                let mut values = Vec::with_capacity(count);
                for i in 0..count {
                    values.push(try!(self.statement.read(i)));
                }
                values
            },
        };
        self.state = Some(try!(self.statement.step()));
        self.values = Some(values);
        Ok(Some(self.values.as_ref().unwrap()))
    }
}

#[inline]
pub fn new<'l>(statement: Statement<'l>) -> Result<Iterator<'l>> {
    Ok(Iterator { state: None, values: None, statement: statement })
}
