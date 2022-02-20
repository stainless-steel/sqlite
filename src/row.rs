use std::collections::HashMap;

use {Error, Result, Value};

/// TODO doc
#[derive(Debug)]
pub struct Row {
    values: Vec<Value>,
    columns_map: HashMap<String, usize>,
}

impl Row {
    /// TODO doc
    #[track_caller]
    pub fn get<T: ValueInto, C: ColumnIndex>(&self, column: C) -> T {
        self.try_get(column).unwrap()
    }

    /// TODO doc
    #[track_caller]
    pub fn try_get<T: ValueInto, C: ColumnIndex>(&self, column: C) -> Result<T> {
        T::into(column.get_value(self)).ok_or_else(|| Error {
            code: None,
            message: Some(format!("column {:?} could not be read", column)),
        })
    }

    // Not part of public API.
    #[doc(hidden)]
    pub fn new(values: Vec<Value>, columns_map: HashMap<String, usize>) -> Self {
        Self {
            values,
            columns_map,
        }
    }
}

/// TODO doc
pub trait ColumnIndex: std::fmt::Debug {
    fn get_value<'a>(&self, row: &'a Row) -> &'a Value;
}

impl ColumnIndex for &str {
    fn get_value<'a>(&self, row: &'a Row) -> &'a Value {
        &row.values[row.columns_map[*self]]
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
