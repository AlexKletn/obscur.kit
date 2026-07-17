//! `MessageKey`: the identity of a translatable message (spec §7.2).

use std::fmt;

/// The key half of §7.2's *key + parameters* abstraction: a stable, opaque name
/// for a message, independent of any language's wording.
///
/// A key is a name, not a default translation. `"file.count"` is a key;
/// `"{n} files"` is an English message that happens to be a string. Keying by the
/// English text is the shortcut that makes English structurally privileged — it
/// forces every other language to be a lookup *from* English and breaks the
/// moment the English wording is edited.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageKey(Box<str>);

impl MessageKey {
    /// Creates a key from its name.
    #[must_use]
    pub fn new(name: impl Into<Box<str>>) -> Self {
        Self(name.into())
    }

    /// Returns the key's name.
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
