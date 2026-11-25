//! `JsonError`: the failure produced when [`parse`](crate::json::parse) cannot
//! read a JSON document.

use std::fmt;

/// A JSON parse failure: a human-readable `message` and the zero-based `char`
/// index into the input where the parser gave up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    /// A human-readable description of what went wrong.
    pub message: String,
    /// The zero-based character index at which parsing failed.
    pub position: usize,
}

impl JsonError {
    /// Creates an error with `message` anchored at character `position`.
    #[must_use]
    pub fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid JSON at {}: {}", self.position, self.message)
    }
}

impl std::error::Error for JsonError {}
