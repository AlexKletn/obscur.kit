//! `JsonValue`: the hand-rolled JSON value model, with builder helpers, typed
//! accessors and a [`Display`] serializer that emits compact, correctly-escaped
//! JSON.

use std::fmt::{self, Write as _};

/// A JSON value: the seven shapes the LSP wire protocol needs.
///
/// Objects preserve insertion order (they are backed by a `Vec` of pairs), so
/// serialization is deterministic — which keeps round-trip tests and on-the-wire
/// output stable.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// The `null` literal.
    Null,
    /// A boolean (`true`/`false`).
    Bool(bool),
    /// A number. JSON has a single numeric type; integers are stored exactly up
    /// to `f64`'s 53-bit mantissa, which covers every LSP integer field.
    Number(f64),
    /// A string (unescaped, stored as native UTF-8).
    String(String),
    /// An array of values.
    Array(Vec<JsonValue>),
    /// An object, as an ordered list of key/value pairs.
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Builds a string value.
    #[must_use]
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Builds an integer number value.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn int(value: i64) -> Self {
        Self::Number(value as f64)
    }

    /// Builds an object from an ordered iterator of key/value pairs.
    #[must_use]
    pub fn object<K: Into<String>>(entries: impl IntoIterator<Item = (K, JsonValue)>) -> Self {
        Self::Object(entries.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// Builds an array from an iterator of values.
    #[must_use]
    pub fn array(items: impl IntoIterator<Item = JsonValue>) -> Self {
        Self::Array(items.into_iter().collect())
    }

    /// Returns the value for `key` if this is an object containing it.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            Self::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Returns the string contents if this is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the number if this is a number.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the number truncated to `i64` if this is a number.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn as_i64(&self) -> Option<i64> {
        self.as_f64().map(|value| value as i64)
    }

    /// Returns the number as a `u32` (clamped to `0..=u32::MAX`) if numeric.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn as_u32(&self) -> Option<u32> {
        self.as_f64()
            .map(|value| value.max(0.0).min(f64::from(u32::MAX)) as u32)
    }

    /// Returns the elements if this is an array.
    #[must_use]
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::Bool(true) => f.write_str("true"),
            Self::Bool(false) => f.write_str("false"),
            Self::Number(value) => write_number(f, *value),
            Self::String(value) => write_string(f, value),
            Self::Array(items) => {
                f.write_char('[')?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        f.write_char(',')?;
                    }
                    write!(f, "{item}")?;
                }
                f.write_char(']')
            }
            Self::Object(entries) => {
                f.write_char('{')?;
                for (index, (key, value)) in entries.iter().enumerate() {
                    if index > 0 {
                        f.write_char(',')?;
                    }
                    write_string(f, key)?;
                    f.write_char(':')?;
                    write!(f, "{value}")?;
                }
                f.write_char('}')
            }
        }
    }
}

/// Writes a number: integral, finite values are emitted without a fractional
/// part (so `1.0` renders as `1`); non-finite values degrade to `null`.
fn write_number(f: &mut fmt::Formatter<'_>, value: f64) -> fmt::Result {
    if !value.is_finite() {
        return f.write_str("null");
    }
    if value.fract() == 0.0 && value.abs() < 9.007_199_254_740_992e15 {
        #[allow(clippy::cast_possible_truncation)]
        return write!(f, "{}", value as i64);
    }
    write!(f, "{value}")
}

/// Writes a JSON string literal with the mandatory escapes.
fn write_string(f: &mut fmt::Formatter<'_>, value: &str) -> fmt::Result {
    f.write_char('"')?;
    for ch in value.chars() {
        match ch {
            '"' => f.write_str("\\\"")?,
            '\\' => f.write_str("\\\\")?,
            '\n' => f.write_str("\\n")?,
            '\r' => f.write_str("\\r")?,
            '\t' => f.write_str("\\t")?,
            '\u{08}' => f.write_str("\\b")?,
            '\u{0c}' => f.write_str("\\f")?,
            c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
            c => f.write_char(c)?,
        }
    }
    f.write_char('"')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_scalars() {
        assert_eq!(JsonValue::Null.to_string(), "null");
        assert_eq!(JsonValue::Bool(true).to_string(), "true");
        assert_eq!(JsonValue::int(42).to_string(), "42");
        assert_eq!(JsonValue::Number(1.0).to_string(), "1");
        assert_eq!(JsonValue::string("hi").to_string(), "\"hi\"");
    }

    #[test]
    fn escapes_control_and_quote_characters() {
        let value = JsonValue::string("a\"b\\c\n\td\u{01}");
        assert_eq!(value.to_string(), "\"a\\\"b\\\\c\\n\\td\\u0001\"");
    }

    #[test]
    fn serializes_nested_object_in_insertion_order() {
        let value = JsonValue::object([
            ("b", JsonValue::int(1)),
            (
                "a",
                JsonValue::array([JsonValue::Bool(false), JsonValue::Null]),
            ),
        ]);
        assert_eq!(value.to_string(), "{\"b\":1,\"a\":[false,null]}");
    }

    #[test]
    fn typed_accessors_read_fields() {
        let value = JsonValue::object([
            ("name", JsonValue::string("App")),
            ("line", JsonValue::int(7)),
        ]);
        assert_eq!(value.get("name").and_then(JsonValue::as_str), Some("App"));
        assert_eq!(value.get("line").and_then(JsonValue::as_u32), Some(7));
        assert!(value.get("missing").is_none());
    }
}
