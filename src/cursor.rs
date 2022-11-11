use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Deref;
use std::rc::Rc;

use error::{Error, Result};
use statement::{Bindable, State, Statement};
use value::Value;

/// An iterator over rows.
pub struct Cursor<'l> {
    statement: Statement<'l>,
    columns: Rc<HashMap<String, usize>>,
    values: Vec<Value>,
    state: Option<State>,
}

/// A row.
#[derive(Debug)]
pub struct Row {
    columns: Rc<HashMap<String, usize>>,
    values: Vec<Value>,
}

/// A type suitable for indexing columns in a row.
pub trait RowColumnIndex: std::fmt::Debug {
    /// Identify the ordinal position.
    fn index(self, row: &Row) -> usize;
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
    /// See `Statement::bind_iter` for further details.
    pub fn bind_iter<T, U>(mut self, value: T) -> Result<Self>
    where
        T: IntoIterator<Item = U>,
        U: Bindable,
    {
        self.state = None;
        self.statement.reset()?;
        self.statement.bind_iter(value)?;
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
        for (index, value) in self.values.iter_mut().enumerate() {
            *value = self.statement.read(index)?;
        }
        self.state = Some(self.statement.next()?);
        Ok(Some(&self.values))
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
        let columns = self.columns.clone();
        self.try_next()
            .map(|row| {
                row.map(|row| Row {
                    columns: columns,
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
    #[inline]
    pub fn get<'l, T, U>(&'l self, column: U) -> T
    where
        T: TryFrom<&'l Value, Error = Error>,
        U: RowColumnIndex,
    {
        self.try_get(column).unwrap()
    }

    /// Try to get the value of a column in the row.
    #[inline]
    pub fn try_get<'l, T, U>(&'l self, column: U) -> Result<T>
    where
        T: TryFrom<&'l Value, Error = Error>,
        U: RowColumnIndex,
    {
        T::try_from(&self.values[column.index(self)])
    }
}

impl Deref for Row {
    type Target = [Value];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl RowColumnIndex for &str {
    #[inline]
    fn index(self, row: &Row) -> usize {
        debug_assert!(row.columns.contains_key(self), "the index is out of range");
        row.columns[self]
    }
}

impl RowColumnIndex for usize {
    #[inline]
    fn index(self, row: &Row) -> usize {
        debug_assert!(self < row.values.len(), "the index is out of range");
        self
    }
}

pub fn new<'l>(statement: Statement<'l>) -> Cursor<'l> {
    let columns = statement
        .column_names()
        .iter()
        .enumerate()
        .map(|(index, name)| (name.to_string(), index))
        .collect();
    let values = vec![Value::Null; statement.column_count()];
    Cursor {
        statement: statement,
        columns: Rc::new(columns),
        values: values,
        state: None,
    }
}
