use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::{Deref, Index};
use std::rc::Rc;

use error::{Error, Result};
use statement::{Bindable, State, Statement};
use value::Value;

/// An iterator over rows.
pub struct Cursor<'l> {
    statement: Statement<'l>,
    values: Vec<Value>,
    state: Option<State>,
}

/// A row.
#[derive(Debug)]
pub struct Row {
    column_mapping: Rc<HashMap<String, usize>>,
    values: Vec<Value>,
}

/// A type suitable for indexing columns in a row.
pub trait RowIndex: std::fmt::Debug {
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

    /// Convert into a prepared statement.
    #[inline]
    pub fn into_statement(self) -> Statement<'l> {
        self.into()
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

impl<'l> From<Cursor<'l>> for Statement<'l> {
    #[inline]
    fn from(cursor: Cursor<'l>) -> Self {
        cursor.statement
    }
}

impl<'l> Iterator for Cursor<'l> {
    type Item = Result<Row>;

    fn next(&mut self) -> Option<Self::Item> {
        let column_mapping = self.statement.column_mapping();
        self.try_next()
            .map(|row| {
                row.map(|row| Row {
                    column_mapping: column_mapping,
                    values: row.to_vec(),
                })
            })
            .transpose()
    }
}

impl Row {
    /// Read the value in a column.
    ///
    /// # Panics
    ///
    /// Panics if the column could not be read.
    #[inline]
    pub fn read<'l, T, U>(&'l self, column: U) -> T
    where
        T: TryFrom<&'l Value, Error = Error>,
        U: RowIndex,
    {
        self.try_read(column).unwrap()
    }

    /// Try to read the value in a column.
    #[inline]
    pub fn try_read<'l, T, U>(&'l self, column: U) -> Result<T>
    where
        T: TryFrom<&'l Value, Error = Error>,
        U: RowIndex,
    {
        T::try_from(&self.values[column.index(self)])
    }
}

impl From<Row> for Vec<Value> {
    #[inline]
    fn from(row: Row) -> Self {
        row.values
    }
}

impl<T> Index<T> for Row
where
    T: RowIndex,
{
    type Output = Value;

    fn index(&self, index: T) -> &Value {
        &self.values[index.index(self)]
    }
}

impl RowIndex for &str {
    #[inline]
    fn index(self, row: &Row) -> usize {
        debug_assert!(
            row.column_mapping.contains_key(self),
            "the index is out of range"
        );
        row.column_mapping[self]
    }
}

impl RowIndex for usize {
    #[inline]
    fn index(self, row: &Row) -> usize {
        debug_assert!(self < row.values.len(), "the index is out of range");
        self
    }
}

pub fn new<'l>(statement: Statement<'l>) -> Cursor<'l> {
    let values = vec![Value::Null; statement.column_count()];
    Cursor {
        statement: statement,
        values: values,
        state: None,
    }
}
