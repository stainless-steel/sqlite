use std::convert::TryFrom;

use crate::error::{Error, Result};

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

/// The type of a value.
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
            Value::Binary(_) => Type::Binary,
            Value::Float(_) => Type::Float,
            Value::Integer(_) => Type::Integer,
            Value::String(_) => Type::String,
            Value::Null => Type::Null,
        }
    }

    /// Try to return the value.
    #[inline]
    pub fn try_into<'l, T>(&'l self) -> Result<T>
    where
        T: TryFrom<&'l Value, Error = Error>,
    {
        T::try_from(self)
    }
}

macro_rules! implement(
    ($type:ty, Null) => {
        impl From<$type> for Value {
            #[inline]
            fn from(_: $type) -> Self {
                Value::Null
            }
        }
    };
    ($type:ty, $value:ident) => {
        impl From<$type> for Value {
            #[inline]
            fn from(value: $type) -> Self {
                Value::$value(value.into())
            }
        }
    };
);

implement!(Vec<u8>, Binary);
implement!(&[u8], Binary);
implement!(f64, Float);
implement!(i64, Integer);
implement!(String, String);
implement!(&str, String);
implement!((), Null);

macro_rules! implement(
    (@value $type:ty, $value:ident) => {
        impl TryFrom<Value> for $type {
            type Error = Error;

            #[inline]
            fn try_from(value: Value) -> Result<Self> {
                if let Value::$value(value) = value {
                    return Ok(value);
                }
                raise!("failed to convert");
            }
        }

        impl TryFrom<Value> for Option<$type> {
            type Error = Error;

            #[inline]
            fn try_from(value: Value) -> Result<Self> {
                if let Value::Null = value {
                    return Ok(None);
                }
                <$type>::try_from(value).and_then(|value| Ok(Some(value)))
            }
        }
    };
    (@reference (), Null) => {
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
    };
    (@reference $type:ty, $value:ident) => {
        impl TryFrom<&Value> for $type {
            type Error = Error;

            #[inline]
            fn try_from(value: &Value) -> Result<Self> {
                if let &Value::$value(value) = value {
                    return Ok(value);
                }
                raise!("failed to convert");
            }
        }

        impl TryFrom<&Value> for Option<$type> {
            type Error = Error;

            #[inline]
            fn try_from(value: &Value) -> Result<Self> {
                if let &Value::Null = value {
                    return Ok(None);
                }
                <$type>::try_from(value).and_then(|value| Ok(Some(value)))
            }
        }
    };
    (@reference-lifetime $type:ty, $value:ident) => {
        impl<'l> TryFrom<&'l Value> for $type {
            type Error = Error;

            #[inline]
            fn try_from(value: &'l Value) -> Result<Self> {
                if let &Value::$value(ref value) = value {
                    return Ok(value);
                }
                raise!("failed to convert");
            }
        }

        impl<'l> TryFrom<&'l Value> for Option<$type> {
            type Error = Error;

            #[inline]
            fn try_from(value: &'l Value) -> Result<Self> {
                if let Value::Null = value {
                    return Ok(None);
                }
                <$type>::try_from(value).and_then(|value| Ok(Some(value)))
            }
        }
    };
);

implement!(@value Vec<u8>, Binary);
implement!(@reference-lifetime &'l [u8], Binary);
implement!(@reference f64, Float);
implement!(@reference i64, Integer);
implement!(@value String, String);
implement!(@reference-lifetime &'l str, String);
implement!(@reference (), Null);

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
    Value: From<T>,
{
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Value::from(value),
            _ => Self::Null,
        }
    }
}
