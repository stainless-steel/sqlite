use statement::{Bindable, State, Statement};
use std::collections::HashMap;
use std::ops::Deref;

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
    /// Bind values to parameters.
    ///
    /// See `Statement::bind` for further details.
    pub fn bind<T: Bindable>(mut self, value: T) -> Result<Self> {
        self.state = None;
        self.statement.reset()?;
        self.statement.bind(value)?;
        Ok(self)
    }

    /// Bind values to parameters via an iterator.
    ///
    /// See `Statement::bind_from` for further details.
    pub fn bind_from<T, U>(mut self, value: T) -> Result<Self>
    where
        T: IntoIterator<Item = U>,
        U: Bindable,
    {
        self.state = None;
        self.statement.reset()?;
        self.statement.bind_from(value)?;
        Ok(self)
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
}

impl<'l> Deref for Cursor<'l> {
    type Target = Statement<'l>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.statement
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
        debug_assert!(row.columns.contains_key(*self), "the index is out of range");
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
