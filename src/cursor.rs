use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::{Deref, Index};
use std::rc::Rc;

use crate::error::{Error, Result};
use crate::statement::{Bindable, State, Statement};
use crate::value::Value;

/// An iterator for a prepared statement.
pub struct Cursor<'l, 'm> {
    column_count: usize,
    statement: &'m mut Statement<'l>,
    poisoned: bool,
}

/// An iterator for a prepared statement with ownership.
pub struct CursorWithOwnership<'l> {
    column_count: usize,
    statement: Statement<'l>,
    poisoned: bool,
}

/// A row.
#[derive(Debug)]
pub struct Row {
    column_mapping: Rc<HashMap<String, usize>>,
    values: Vec<Value>,
}

/// A type suitable for indexing columns in a row.
pub trait RowIndex: std::fmt::Debug {
    /// Check to see if the row contains a Column
    fn contains(&self, row: &Row) -> bool;

    /// Identify the ordinal position.
    ///
    /// The first column has index 0.
    fn index(self, row: &Row) -> usize;
}

macro_rules! implement(
    ($type:ident<$($lifetime:lifetime),+>) => {
        impl<$($lifetime),+> $type<$($lifetime),+> {
            /// Bind values to parameters.
            ///
            /// In case of integer indices, the first parameter has index 1. See `Statement::bind`
            /// for further details.
            pub fn bind<T: Bindable>(self, value: T) -> Result<Self> {
                #[allow(unused_mut)]
                let mut cursor = self.reset()?;
                cursor.statement.bind(value)?;
                Ok(cursor)
            }

            /// Bind values to parameters via an iterator.
            ///
            /// See `Statement::bind_iter` for further details.
            #[allow(unused_mut)]
            pub fn bind_iter<T, U>(self, value: T) -> Result<Self>
            where
                T: IntoIterator<Item = U>,
                U: Bindable,
            {
                let mut cursor = self.reset()?;
                cursor.statement.bind_iter(value)?;
                Ok(cursor)
            }

            /// Reset the internal state.
            #[allow(unused_mut)]
            pub fn reset(mut self) -> Result<Self> {
                self.statement.reset()?;
                self.poisoned = false;
                Ok(self)
            }

            /// Advance to the next row and read all columns.
            pub fn try_next(&mut self) -> Result<Option<Vec<Value>>> {
                if self.statement.next()? == State::Done {
                    return Ok(None);
                }
                let mut values = Vec::with_capacity(self.column_count);
                for index in 0..self.column_count {
                    values.push(self.statement.read(index)?);
                }
                Ok(Some(values))
            }
        }

        impl<$($lifetime),+> Deref for $type<$($lifetime),+> {
            type Target = Statement<'l>;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.statement
            }
        }

        impl<$($lifetime),+> Iterator for $type<$($lifetime),+> {
            type Item = Result<Row>;

            fn next(&mut self) -> Option<Self::Item> {
                if self.poisoned {
                    return None;
                }
                match self.try_next() {
                    Ok(value) => {
                        value.map(|values| Ok(Row {
                            column_mapping: self.statement.column_mapping(),
                            values,
                        }))
                    }
                    Err(error) => {
                        self.poisoned = true;
                        Some(Err(error))
                    }
                }
            }
        }
    }
);

implement!(Cursor<'l, 'm>);
implement!(CursorWithOwnership<'l>);

impl<'l> From<CursorWithOwnership<'l>> for Statement<'l> {
    #[inline]
    fn from(cursor: CursorWithOwnership<'l>) -> Self {
        cursor.statement
    }
}

impl Row {
    /// Read the value in a column.
    ///
    /// In case of integer indices, the first column has index 0.
    ///
    /// # Panics
    ///
    /// Panics if the column can not be read.
    #[inline]
    pub fn read<'l, T, U>(&'l self, column: U) -> T
    where
        T: TryFrom<&'l Value, Error = Error>,
        U: RowIndex,
    {
        self.try_read(column).unwrap()
    }

    /// Take the value from a column.
    ///
    /// In case of integer indices, the first column has index 0. Any subsequent invocation will
    /// result in `Value::Null`.
    #[inline]
    pub fn take<U>(&mut self, column: U) -> Value
    where
        U: RowIndex,
    {
        let index = column.index(self);
        std::mem::take(&mut self.values[index])
    }

    /// Try to read the value in a column.
    ///
    /// In case of integer indices, the first column has index 0.
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
            "the index is out of range",
        );
        row.column_mapping[self]
    }

    fn contains(&self, row: &Row) -> bool {
        row.column_mapping.contains_key(*self)
    }
}

impl RowIndex for usize {
    #[inline]
    fn index(self, row: &Row) -> usize {
        debug_assert!(self < row.values.len(), "the index is out of range");
        self
    }

    fn contains(&self, row: &Row) -> bool {
        self < &row.values.len()
    }
}

pub fn new<'l, 'm>(statement: &'m mut Statement<'l>) -> Cursor<'l, 'm> {
    Cursor {
        column_count: statement.column_count(),
        statement,
        poisoned: false,
    }
}

pub fn new_with_ownership(statement: Statement<'_>) -> CursorWithOwnership<'_> {
    CursorWithOwnership {
        column_count: statement.column_count(),
        statement,
        poisoned: false,
    }
}
