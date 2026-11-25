#![doc = include_str!("../../docs/api/scope/scoped_ident.doc/_module.md")]

use crate::scope::ScopeId;
use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/scope/scoped_ident.doc/ScopedIdent.md")]
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
        use std::collections::HashSet;

        let a = ScopedIdent::new(ScopeId::from_raw(1), InternedSymbol::from_raw(1));
        let b = ScopedIdent::new(ScopeId::from_raw(1), InternedSymbol::from_raw(1));
        assert_eq!(a, b);

        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }
}
