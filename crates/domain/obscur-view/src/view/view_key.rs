#![doc = include_str!("view_key.doc/_module.md")]

use obscur_identity::StableKey;

#[doc = include_str!("view_key.doc/ViewKey.md")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewKey(StableKey);

impl ViewKey {
    #[doc = include_str!("view_key.doc/new.md")]
    #[must_use]
    pub const fn new(key: StableKey) -> Self {
        Self(key)
    }

    #[doc = include_str!("view_key.doc/stable_key.md")]
    #[must_use]
    pub const fn stable_key(&self) -> &StableKey {
        &self.0
    }

    #[doc = include_str!("view_key.doc/into_stable_key.md")]
    #[must_use]
    pub fn into_stable_key(self) -> StableKey {
        self.0
    }
}

impl From<StableKey> for ViewKey {
    fn from(key: StableKey) -> Self {
        Self(key)
    }
}

impl From<u64> for ViewKey {
    fn from(value: u64) -> Self {
        Self(StableKey::from(value))
    }
}

impl From<&str> for ViewKey {
    fn from(value: &str) -> Self {
        Self(StableKey::from(value))
    }
}

#[doc = include_str!("view_key.doc/From.md")]
impl From<String> for ViewKey {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_u64_builds_index_key() {
        let key = ViewKey::from(7u64);
        assert_eq!(key.stable_key(), &StableKey::Index(7));
    }

    #[test]
    fn from_str_builds_name_key() {
        let key = ViewKey::from("row");
        assert_eq!(key, ViewKey::new(StableKey::from("row")));
    }

    #[test]
    fn into_stable_key_round_trips() {
        let key = ViewKey::from(3u64);
        assert_eq!(key.into_stable_key(), StableKey::Index(3));
    }
}
