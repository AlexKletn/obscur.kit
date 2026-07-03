//! Author-provided identity for keyed children.

/// Explicit identity for a child within a keyed collection, used to preserve
/// retained identity across reconciliation (spec §2.2, §2.11).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StableKey {
    /// A positional key derived from a stable index.
    Index(u64),
    /// A named key provided by the author.
    Name(Box<str>),
    /// An interned symbolic key.
    Symbol(super::InternedSymbol),
}

impl From<u64> for StableKey {
    fn from(value: u64) -> Self {
        Self::Index(value)
    }
}

impl From<&str> for StableKey {
    fn from(value: &str) -> Self {
        Self::Name(value.into())
    }
}
