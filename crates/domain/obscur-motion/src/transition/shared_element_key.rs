#[doc = include_str!("../../docs/api/transition/shared_element_transition.doc/SharedElementKey.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SharedElementKey(Box<str>);

impl SharedElementKey {
    /// Creates a key from any string-like tag (spec §4.17).
    #[must_use]
    pub fn new(tag: impl Into<Box<str>>) -> Self {
        Self(tag.into())
    }

    /// Returns the tag as a string slice (spec §4.17).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SharedElementKey;

    #[test]
    fn key_round_trips_tag() {
        let key = SharedElementKey::new("avatar");
        assert_eq!(key.as_str(), "avatar");
    }
}
