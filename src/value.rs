use std::convert::TryFrom;

use {Error, Result};

/// A value.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Binary data.
    Binary(Vec<u8>),
    /// A floating-point number.
    Float(f64),
    /// An integer number.
    Integer(i64),
    /// A string.
    String(String),
    /// A null value.
    Null,
}

/// A type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    /// The binary type.
    Binary,
    /// The floating-point type.
    Float,
    /// The integer type.
    Integer,
    /// The string type.
    String,
    /// The null type.
    Null,
}

impl Value {
    /// Return the type.
    pub fn kind(&self) -> Type {
        match self {
            &Value::Binary(_) => Type::Binary,
            &Value::Float(_) => Type::Float,
            &Value::Integer(_) => Type::Integer,
            &Value::String(_) => Type::String,
            &Value::Null => Type::Null,
        }
    }

    #[inline]
    pub fn try_into<'l, T>(&'l self) -> Result<T>
    where
        T: TryFrom<&'l Value, Error = Error>,
    {
        T::try_from(self)
    }
}

impl From<Vec<u8>> for Value {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Value::Binary(value)
    }
}

impl From<&[u8]> for Value {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Value::Binary(value.into())
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<i64> for Value {
    #[inline]
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

impl From<String> for Value {
    #[inline]
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    #[inline]
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}

impl From<()> for Value {
    #[inline]
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl TryFrom<Value> for Vec<u8> {
    type Error = Error;

    #[inline]
    fn try_from(value: Value) -> Result<Self> {
        if let Value::Binary(value) = value {
            return Ok(value);
        }
        raise!("failed to convert");
    }
}

impl<'l> TryFrom<&'l Value> for &'l [u8] {
    type Error = Error;

    #[inline]
    fn try_from(value: &'l Value) -> Result<Self> {
        if let &Value::Binary(ref value) = value {
            return Ok(value);
        }
        raise!("failed to convert");
    }
}

impl TryFrom<&Value> for f64 {
    type Error = Error;

    #[inline]
    fn try_from(value: &Value) -> Result<Self> {
        if let &Value::Float(value) = value {
            return Ok(value);
        }
        raise!("failed to convert");
    }
}

impl TryFrom<&Value> for i64 {
    type Error = Error;

    #[inline]
    fn try_from(value: &Value) -> Result<Self> {
        if let &Value::Integer(value) = value {
            return Ok(value);
        }
        raise!("failed to convert");
    }
}

impl TryFrom<Value> for String {
    type Error = Error;

    #[inline]
    fn try_from(value: Value) -> Result<Self> {
        if let Value::String(value) = value {
            return Ok(value);
        }
        raise!("failed to convert");
    }
}

impl<'l> TryFrom<&'l Value> for &'l str {
    type Error = Error;

    #[inline]
    fn try_from(value: &'l Value) -> Result<Self> {
        if let &Value::String(ref value) = value {
            return Ok(value);
        }
        raise!("failed to convert");
    }
}

impl TryFrom<&Value> for () {
    type Error = Error;

    #[inline]
    fn try_from(value: &Value) -> Result<Self> {
        if let &Value::Null = value {
            return Ok(());
        }
        raise!("failed to convert");
    }
}
