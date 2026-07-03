//! A local identifier isolated to a style scope.
//!
//! CSS-module isolation (§4.15) means a component's internal class names do not
//! need global uniqueness: two different modules may both use `.item` without
//! colliding. [`ScopedIdent`] models that by pairing a local
//! [`InternedSymbol`] with the [`ScopeId`] it belongs to, so identity is the
//! *combination* of scope and local name rather than the name alone.

use crate::scope::ScopeId;
use obscur_identity::InternedSymbol;

/// A local identifier (for example a class name) qualified by the scope that
/// owns it (§4.15).
///
/// Because equality and hashing consider *both* fields, `.item` in one module
/// and `.item` in another are distinct [`ScopedIdent`]s — this is how module
/// isolation avoids requiring local identifiers to be globally unique.
/// `Copy + Eq + Hash` so it can key maps directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopedIdent {
    scope: ScopeId,
    local: InternedSymbol,
}

impl ScopedIdent {
    /// Creates a scoped identifier binding `local` to `scope`.
    #[must_use]
    pub const fn new(scope: ScopeId, local: InternedSymbol) -> Self {
        Self { scope, local }
    }

    /// Returns the scope that owns this identifier.
    #[must_use]
    pub const fn scope(self) -> ScopeId {
        self.scope
    }

    /// Returns the local (unqualified) identifier.
    #[must_use]
    pub const fn local(self) -> InternedSymbol {
        self.local
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessors_round_trip() {
        let scope = ScopeId::from_raw(3);
        let local = InternedSymbol::from_raw(9);
        let ident = ScopedIdent::new(scope, local);
        assert_eq!(ident.scope(), scope);
        assert_eq!(ident.local(), local);
    }

    #[test]
    fn same_local_in_different_scopes_is_distinct() {
        let local = InternedSymbol::from_raw(1);
        let a = ScopedIdent::new(ScopeId::from_raw(1), local);
        let b = ScopedIdent::new(ScopeId::from_raw(2), local);
        assert_ne!(a, b);
    }

    #[test]
    fn equal_only_when_both_match() {
        let a = ScopedIdent::new(ScopeId::from_raw(1), InternedSymbol::from_raw(1));
        let b = ScopedIdent::new(ScopeId::from_raw(1), InternedSymbol::from_raw(1));
        assert_eq!(a, b);

        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }
}
