#![doc = include_str!("../../docs/api/scope/module_scope.doc/_module.md")]

use crate::scope::{ScopeId, ScopedIdent};
use obscur_identity::InternedSymbol;

#[doc = include_str!("../../docs/api/scope/module_scope.doc/ModuleScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleScope {
    scope: ScopeId,
}

impl ModuleScope {
    /// Creates module isolation rooted at the given scope.
    #[must_use]
    pub const fn new(scope: ScopeId) -> Self {
        Self { scope }
    }

    /// Returns the scope this module isolates identifiers into.
    #[must_use]
    pub const fn scope(self) -> ScopeId {
        self.scope
    }

    #[doc = include_str!("../../docs/api/scope/module_scope.doc/localize.md")]
    #[must_use]
    pub const fn localize(self, local: InternedSymbol) -> ScopedIdent {
        ScopedIdent::new(self.scope, local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accessor_round_trips() {
        let module = ModuleScope::new(ScopeId::from_raw(5));
        assert_eq!(module.scope(), ScopeId::from_raw(5));
    }

    #[test]
    fn localizes_into_its_scope() {
        let module = ModuleScope::new(ScopeId::from_raw(5));
        let ident = module.localize(InternedSymbol::from_raw(2));
        assert_eq!(ident.scope(), ScopeId::from_raw(5));
        assert_eq!(ident.local(), InternedSymbol::from_raw(2));
    }

    #[test]
    fn same_local_in_two_modules_does_not_collide() {
        let local = InternedSymbol::from_raw(1);
        let a = ModuleScope::new(ScopeId::from_raw(1)).localize(local);
        let b = ModuleScope::new(ScopeId::from_raw(2)).localize(local);
        assert_ne!(a, b);
    }
}
