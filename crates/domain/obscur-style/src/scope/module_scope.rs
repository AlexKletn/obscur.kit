//! CSS-module isolation applied at a component / scope root.
//!
//! Per §4.15, CSS-module mode applies only at a component or scope root. It
//! isolates the local identifiers declared inside that scope — internal class
//! names need not be globally unique — while keeping the normal cascade,
//! inheritance and custom-property (variable) behaviour intact inside the
//! scope. A [`ModuleScope`] is the operation that performs that local-identifier
//! isolation: it maps a bare local name into a [`ScopedIdent`] qualified by its
//! scope so it cannot clash with another module's same-named identifier.

use crate::scope::{ScopeId, ScopedIdent};
use obscur_identity::InternedSymbol;

/// CSS-module isolation for a single scope root (§4.15).
///
/// It carries only the [`ScopeId`] it localizes into; the actual selector
/// rewriting lives in the matcher / `obscur-css`. `Copy + Eq + Hash`.
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

    /// Maps a local identifier into this scope, producing a [`ScopedIdent`].
    ///
    /// This is how module isolation avoids requiring global uniqueness: the same
    /// `local` mapped through two different [`ModuleScope`]s yields two distinct
    /// [`ScopedIdent`]s that never collide (§4.15).
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
