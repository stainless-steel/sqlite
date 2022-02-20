use std::collections::HashMap;

use {Error, Result, Statement, Value};

/// TODO doc
#[derive(Debug)]
pub struct Row {
    row: Vec<Value>,
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

    /// TODO doc
    pub fn read<'l>(statement: &Statement<'l>) -> Result<Self> {
        let count = statement.column_count();
        let mut row = Vec::with_capacity(count);
        for i in 0..count {
            row.push(statement.read(i)?);
        }

        let columns_map = (0..statement.column_count())
            .map(|i| (statement.column_name(i).to_string(), i))
            .collect();

        Ok(Self { row, columns_map })
    }
}

/// TODO doc
pub trait ColumnIndex: std::fmt::Debug {
    fn get_value<'a>(&self, row: &'a Row) -> &'a Value;
}

impl ColumnIndex for &str {
    fn get_value<'a>(&self, row: &'a Row) -> &'a Value {
        &row.row[row.columns_map[*self]]
    }
}

impl ColumnIndex for usize {
    fn get_value<'a>(&self, row: &'a Row) -> &'a Value {
        &row.row[*self]
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
        value.as_string().map(|s| s.to_string())
    }
}

impl ValueInto for Vec<u8> {
    fn into(value: &Value) -> Option<Self> {
        value.as_binary().map(|s| s.to_vec())
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
