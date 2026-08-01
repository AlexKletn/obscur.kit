#![doc = include_str!("error.doc/_module.md")]

use std::fmt;

#[doc = include_str!("error.doc/JsonError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    #[doc = include_str!("error.doc/message.md")]
    pub message: String,
    #[doc = include_str!("error.doc/position.md")]
    pub position: usize,
}

impl JsonError {
    #[doc = include_str!("error.doc/new.md")]
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
