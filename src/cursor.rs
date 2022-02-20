use ffi;
use statement::{State, Statement};
use std::collections::HashMap;

use {Result, Value};

/// An iterator over rows.
pub struct Cursor<'l> {
    state: Option<State>,
    values: Option<Vec<Value>>,
    statement: Statement<'l>,
    columns: Option<HashMap<String, usize>>,
}

impl<'l> Cursor<'l> {
    /// Bind values to parameters by index.
    ///
    /// The index of each value is assumed to be the value’s position in the array.
    pub fn bind(mut self, values: &[Value]) -> Result<Self> {
        self.state = None;
        self.statement.reset()?;
        for (i, value) in values.iter().enumerate() {
            self.statement = self.statement.bind(i + 1, value)?;
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
    pub fn bind_by_name<S, V>(mut self, values: V) -> Result<Self>
    where
        S: AsRef<str>,
        V: IntoIterator<Item = (S, Value)>,
    {
        self.state = None;
        self.statement.reset()?;
        for (name, value) in values {
            if let Some(i) = self.statement.parameter_index(name.as_ref())? {
                self.statement = self.statement.bind(i, &value)?;
            }
        }
        Ok(self)
    }

    /// Return the number of columns.
    #[inline]
    pub fn column_count(&self) -> usize {
        self.statement.column_count()
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

    #[deprecated(since = "0.26.0", note = "Please use `column_count` instead.")]
    pub fn count(&self) -> usize {
        self.column_count()
    }
}

impl<'l> Iterator for Cursor<'l> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        let columns = match self.columns.clone() {
            Some(columns) => columns,
            None => {
                self.columns = Some(
                    (0..self.statement.column_count())
                        .map(|i| (self.statement.column_name(i).to_string(), i))
                        .collect(),
                );
                self.columns.clone().unwrap()
            }
        };

        self.try_next()
            .map(|option| {
                option.map(|values| Row {
                    values: values.to_vec(),
                    columns,
                })
            })
            .transpose()
    }
}

#[inline]
pub fn new<'l>(statement: Statement<'l>) -> Cursor<'l> {
    Cursor {
        state: None,
        values: None,
        statement: statement,
        columns: None,
    }
}

/// TODO doc
#[derive(Debug)]
pub struct Row {
    values: Vec<Value>,
    columns: HashMap<String, usize>,
}

impl Row {
    /// TODO doc
    #[track_caller]
    #[inline]
    pub fn get<T: ValueInto, C: ColumnIndex>(&self, column: C) -> T {
        self.try_get(column).unwrap()
    }

    /// TODO doc
    #[track_caller]
    #[inline]
    pub fn try_get<T: ValueInto, C: ColumnIndex>(&self, column: C) -> Result<T> {
        match T::into(column.get_value(self)) {
            Some(value) => Ok(value),
            None => raise!("column {:?} could not be read", column),
        }
    }
}

/// TODO doc
pub trait ColumnIndex: std::fmt::Debug {
    fn get_value<'a>(&self, row: &'a Row) -> &'a Value;
}

impl ColumnIndex for &str {
    fn get_value<'a>(&self, row: &'a Row) -> &'a Value {
        &row.values[row.columns[*self]]
    }
}

impl ColumnIndex for usize {
    fn get_value<'a>(&self, row: &'a Row) -> &'a Value {
        &row.values[*self]
    }
}

/// TODO doc
pub trait ValueInto: Sized {
    fn into(value: &Value) -> Option<Self>;
}

impl ValueInto for Value {
    fn into(value: &Value) -> Option<Self> {
        Some(value.clone())
    }
}

impl ValueInto for i64 {
    fn into(value: &Value) -> Option<Self> {
        value.as_integer()
    }
}

impl ValueInto for f64 {
    fn into(value: &Value) -> Option<Self> {
        value.as_float()
    }
}

impl ValueInto for String {
    fn into(value: &Value) -> Option<Self> {
        value.as_string().map(|slice| slice.to_string())
    }
}

impl ValueInto for Vec<u8> {
    fn into(value: &Value) -> Option<Self> {
        value.as_binary().map(|bytes| bytes.to_vec())
    }
}

impl<T: ValueInto> ValueInto for Option<T> {
    fn into(value: &Value) -> Option<Self> {
        match value {
            Value::Null => Some(None),
            _ => T::into(value).map(Some),
        }
    }
}
