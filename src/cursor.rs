use ffi;
use statement::{State, Statement};
use std::collections::HashMap;

use {Result, Value};

/// An iterator over rows.
pub struct Cursor<'l> {
    state: Option<State>,
    columns: Option<HashMap<String, usize>>,
    values: Option<Vec<Value>>,
    statement: Statement<'l>,
}

/// A row.
#[derive(Debug)]
pub struct Row {
    columns: HashMap<String, usize>,
    values: Vec<Value>,
}

/// A type suitable for indexing columns.
pub trait ColumnIndex: std::fmt::Debug {
    fn get<'l>(&self, row: &'l Row) -> &'l Value;
}

/// A type that values can be converted into.
pub trait ValueInto: Sized {
    fn into(value: &Value) -> Option<Self>;
}

impl<'l> Cursor<'l> {
    /// Bind values to parameters by index.
    ///
    /// The index of each value is assumed to be the value’s position in the
    /// array.
    pub fn bind(mut self, values: &[Value]) -> Result<Self> {
        self.state = None;
        self.statement.reset()?;
        for (i, value) in values.iter().enumerate() {
            self.statement.bind(i + 1, value)?;
        }
        Ok(self)
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
    /// let mut cursor = statement
    ///     .into_cursor()
    ///     .bind_by_name(vec![
    ///         (":name", Value::String("Bob".to_owned())),
    ///         (":id", Value::Integer(42)),
    ///     ])?;
    /// cursor.try_next()?;
    /// # Ok::<(), sqlite::Error>(())
    /// ```
    pub fn bind_by_name<T, U>(mut self, values: U) -> Result<Self>
    where
        T: AsRef<str>,
        U: IntoIterator<Item = (T, Value)>,
    {
        self.state = None;
        self.statement.reset()?;
        for (name, value) in values {
            if let Some(i) = self.statement.parameter_index(name.as_ref())? {
                self.statement.bind(i, &value)?;
            }
        }
        Ok(self)
    }

    /// Return the number of columns.
    #[inline]
    pub fn column_count(&self) -> usize {
        self.statement.column_count()
    }

    /// Return column names.
    #[inline]
    pub fn column_names(&self) -> Vec<&str> {
        self.statement.column_names()
    }

    /// Advance to the next row and read all columns.
    pub fn try_next(&mut self) -> Result<Option<&[Value]>> {
        match self.state {
            Some(State::Row) => {}
            Some(State::Done) => return Ok(None),
            _ => {
                self.state = Some(self.statement.next()?);
                return self.try_next();
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
                let count = self.statement.column_count();
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

impl<'l> Iterator for Cursor<'l> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        let columns = match self.columns.clone() {
            Some(columns) => columns,
            None => {
                self.columns = Some(
                    self.column_names()
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (name.to_string(), i))
                        .collect(),
                );
                self.columns.clone().unwrap()
            }
        };
        self.try_next()
            .map(|row| {
                row.map(|row| Row {
                    columns,
                    values: row.to_vec(),
                })
            })
            .transpose()
    }
}

impl Row {
    /// Get the value of a column in the row.
    ///
    /// # Panics
    ///
    /// Panics if the column could not be read.
    #[track_caller]
    #[inline]
    pub fn get<T: ValueInto, U: ColumnIndex>(&self, column: U) -> T {
        self.try_get(column).unwrap()
    }

    /// Try to get the value of a column in the row.
    ///
    /// It returns an error if the column could not be read.
    #[track_caller]
    #[inline]
    pub fn try_get<T: ValueInto, U: ColumnIndex>(&self, column: U) -> Result<T> {
        match T::into(column.get(self)) {
            Some(value) => Ok(value),
            None => raise!("column {:?} could not be read", column),
        }
    }
}

impl ColumnIndex for &str {
    #[inline]
    fn get<'l>(&self, row: &'l Row) -> &'l Value {
        debug_assert!(row.columns.contains_key(*self), "the index is out of range",);
        &row.values[row.columns[*self]]
    }
}

impl ColumnIndex for usize {
    #[inline]
    fn get<'l>(&self, row: &'l Row) -> &'l Value {
        debug_assert!(*self < row.values.len(), "the index is out of range");
        &row.values[*self]
    }
}

impl ValueInto for Value {
    #[inline]
    fn into(value: &Value) -> Option<Self> {
        Some(value.clone())
    }
}

impl ValueInto for i64 {
    #[inline]
    fn into(value: &Value) -> Option<Self> {
        value.as_integer()
    }
}

impl ValueInto for f64 {
    #[inline]
    fn into(value: &Value) -> Option<Self> {
        value.as_float()
    }
}

impl ValueInto for String {
    #[inline]
    fn into(value: &Value) -> Option<Self> {
        value.as_string().map(|slice| slice.to_string())
    }
}

impl ValueInto for Vec<u8> {
    #[inline]
    fn into(value: &Value) -> Option<Self> {
        value.as_binary().map(|bytes| bytes.to_vec())
    }
}

impl<T: ValueInto> ValueInto for Option<T> {
    #[inline]
    fn into(value: &Value) -> Option<Self> {
        match value {
            Value::Null => Some(None),
            _ => T::into(value).map(Some),
        }
    }
}

#[inline]
pub fn new<'l>(statement: Statement<'l>) -> Cursor<'l> {
    Cursor {
        state: None,
        columns: None,
        values: None,
        statement: statement,
    }
}
