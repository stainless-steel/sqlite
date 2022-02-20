use ffi;
use statement::{State, Statement};
use {Result, Row, Value};

/// An iterator over rows.
pub struct Cursor<'l> {
    state: Option<State>,
    statement: Statement<'l>,
}

impl<'l> Cursor<'l> {
    /// Bind values to parameters by index.
    ///
    /// The index of each value is assumed to be the value’s position in the array.
    pub fn bind(&mut self, values: &[Value]) -> Result<()> {
        self.state = None;
        self.statement.reset()?;
        for (i, value) in values.iter().enumerate() {
            self.statement.bind(i + 1, value)?;
        }
        Ok(())
    }

    /// Bind values to parameters by name.
    ///
    /// Parameters that are not part of the statement will be ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sqlite::Value;
    /// # let connection = sqlite::open(":memory:").unwrap();
    /// # connection.execute("CREATE TABLE users (id INTEGER, name STRING)");
    /// let statement = connection.prepare("INSERT INTO users VALUES (:id, :name)")?;
    /// let mut cursor = statement.into_cursor();
    /// cursor.bind_by_name(vec![
    ///     (":name", Value::String("Bob".to_owned())),
    ///     (":id", Value::Integer(42)),
    /// ])?;
    /// cursor.next().transpose()?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    pub fn bind_by_name<S, V>(&mut self, values: V) -> Result<()>
    where
        S: AsRef<str>,
        V: IntoIterator<Item = (S, Value)>,
    {
        self.state = None;
        self.statement.reset()?;
        for (name, value) in values {
            if let Some(i) = self.statement.parameter_index(name.as_ref())? {
                self.statement.bind(i, &value)?;
            }
        }
        Ok(())
    }

    /// Return the number of columns.
    #[inline]
    pub fn column_count(&self) -> usize {
        self.statement.column_count()
    }

    fn try_next(&mut self) -> Result<Option<Row>> {
        match self.state {
            Some(State::Row) => {}
            Some(State::Done) => return Ok(None),
            _ => {
                self.state = Some(self.statement.next()?);
                return self.try_next();
            }
        }
        let row = Row::read(&self.statement)?;
        self.state = Some(self.statement.next()?);
        Ok(Some(row))
    }

    /// Return the raw pointer.
    #[inline]
    pub fn as_raw(&self) -> *mut ffi::sqlite3_stmt {
        self.statement.as_raw()
    }

    #[deprecated(since = "0.26.0", note = "Please use `column_count` instead.")]
    pub fn count(&self) -> usize {
        self.column_count()
    }
}

impl<'l> Iterator for Cursor<'l> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}

#[inline]
pub fn new<'l>(statement: Statement<'l>) -> Cursor<'l> {
    Cursor {
        state: None,
        statement: statement,
    }
}
