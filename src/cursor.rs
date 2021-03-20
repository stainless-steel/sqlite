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
        self.statement.reset()?;
        for (i, value) in values.iter().enumerate() {
            self.statement.bind(i + 1, value)?;
        }
        Ok(())
    }

    /// Bind values to all named parameters.
    ///
    /// Any parameters provided that are not part of the statement will be ignored.
    ///
    /// # Examples
    /// ```
    /// # use sqlite::Value;
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (id INTEGER, name STRING)");
    /// let statement = connection.prepare("INSERT INTO users VALUES (:id, :name)")?;
    /// let mut cursor = statement.cursor();
    /// cursor.bind_params(vec![
    ///     (":name", Value::String("Bob".to_owned())),
    ///     (":id", Value::Integer(42)),
    /// ])?;
    /// cursor.next()?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    pub fn bind_params<S, V>(&mut self, values: V) -> Result<()>
    where
        S: AsRef<str>,
        V: IntoIterator<Item = (S, Value)>,
    {
        self.state = None;
        self.statement.reset()?;
        for (param, value) in values {
            if let Some(i) = self.statement.parameter_index(param.as_ref())? {
                self.statement.bind(i.get(), &value)?;
            }
        }
        Ok(())
    }

    /// Return the number of columns.
    #[inline]
    pub fn count(&self) -> usize {
        self.statement.count()
    }

    /// Advance to the next row and read all columns.
    pub fn next(&mut self) -> Result<Option<&[Value]>> {
        match self.state {
            Some(State::Row) => {}
            Some(State::Done) => return Ok(None),
            _ => {
                self.state = Some(self.statement.next()?);
                return self.next();
            }
        }
        self.values = match self.values.take() {
            Some(mut values) => {
                for (i, value) in values.iter_mut().enumerate() {
                    *value = self.statement.read(i)?;
                }
                Some(values)
            }
            _ => {
                let count = self.statement.count();
                let mut values = Vec::with_capacity(count);
                for i in 0..count {
                    values.push(self.statement.read(i)?);
                }
                Some(values)
            }
        };
        self.state = Some(self.statement.next()?);
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
