#![doc = include_str!("message_key.doc/_module.md")]

use std::fmt;

#[doc = include_str!("message_key.doc/MessageKey.md")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageKey(Box<str>);

impl MessageKey {
    #[doc = include_str!("message_key.doc/new.md")]
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self(name.into())
    }

    #[doc = include_str!("message_key.doc/as_str.md")]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for MessageKey {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl fmt::Display for MessageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carries_its_name() {
        assert_eq!(MessageKey::new("file.count").as_str(), "file.count");
        assert_eq!(MessageKey::from("file.count").to_string(), "file.count");
    }

    #[test]
    fn keys_compare_by_name() {
        assert_eq!(MessageKey::new("a"), MessageKey::from("a"));
        assert_ne!(MessageKey::new("a"), MessageKey::new("b"));
    }
}
