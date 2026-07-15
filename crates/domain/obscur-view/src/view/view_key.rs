//! `ViewKey`: explicit stable identity for a keyed child.

use obscur_identity::StableKey;

/// Explicit, author-provided stable identity for a keyed child in the desired
/// view tree.
///
/// Keyed collections (`For`, conditionals, spec §2.11) require a mandatory
/// stable key so a child preserves its retained identity across reconciliation
/// even when siblings are inserted, removed, or reordered (spec §2.2). The key
/// lives on the desired node as plain data; it never references a retained-tree
/// `NodeId`.
///
/// This is a thin newtype over [`StableKey`] so the view layer owns its own
/// vocabulary type while reusing the foundation identity contract.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewKey(StableKey);

impl ViewKey {
    /// Wraps a foundation [`StableKey`] as a `ViewKey`.
    #[must_use]
    pub const fn new(key: StableKey) -> Self {
        Self(key)
    }

    /// Returns the underlying [`StableKey`].
    #[must_use]
    pub const fn stable_key(&self) -> &StableKey {
        &self.0
    }

    /// Consumes this `ViewKey`, returning the underlying [`StableKey`].
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

/// The owned twin of the [`&str`](str) conversion.
///
/// It exists so a caller holding a `String` key needs no `.as_str()` — which
/// matters most to generated code: `.okui`'s `key={item.id}` emits one expression
/// whatever the field's type is, and `u64` and `String` keys have to travel the
/// same path or the emitter would need to know the type it is about to hand to
/// rustc.
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
