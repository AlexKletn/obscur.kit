//! Opaque identifier for a style scope.
//!
//! A [`ScopeId`] names a style scope (the boundary that scopes a component's or
//! module's CSS, per §4.15) without carrying any behaviour. It is a lightweight
//! newtype over an integer so it can key maps and be compared and ordered
//! cheaply. The behaviour of a scope (isolation, boundary-crossing) lives on
//! [`crate::scope::StyleScope`]; this type is only its identity.

/// An opaque identifier for a style scope (§4.15).
///
/// This is a newtype over a `u32`: values are only meaningful relative to the
/// allocator that hands them out. `Copy + Eq + Hash + Ord` so it can key maps
/// and order deterministically.
///
/// [`ScopeId::GLOBAL`] is a reserved value denoting the root / global
/// application scope; it is handed out by convention rather than being special
/// to this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScopeId(u32);

impl ScopeId {
    /// The reserved identifier for the root / global application scope.
    ///
    /// The global scope performs no isolation, so it is convenient to give it a
    /// well-known, stable value that allocators can reserve.
    pub const GLOBAL: ScopeId = ScopeId(0);

    /// Wraps a raw integer as a [`ScopeId`].
    ///
    /// The caller is responsible for the integer being a meaningful scope
    /// identifier within its allocation domain.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw integer backing this identifier.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns `true` when this is the reserved global scope identifier.
    #[must_use]
    pub const fn is_global(self) -> bool {
        self.0 == Self::GLOBAL.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_raw() {
        assert_eq!(ScopeId::from_raw(42).raw(), 42);
    }

    #[test]
    fn global_is_reserved_zero() {
        assert!(ScopeId::GLOBAL.is_global());
        assert!(!ScopeId::from_raw(1).is_global());
    }

    #[test]
    fn is_hashable_and_ord() {
        use std::collections::BTreeSet;
        let mut set = BTreeSet::new();
        set.insert(ScopeId::from_raw(2));
        set.insert(ScopeId::from_raw(1));
        set.insert(ScopeId::from_raw(1));
        assert_eq!(set.len(), 2);
        assert_eq!(set.iter().next(), Some(&ScopeId::from_raw(1)));
    }
}
