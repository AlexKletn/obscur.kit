//! A style scope: the boundary that scopes a component's or module's CSS.
//!
//! Per §4.15, global stylesheets are allowed but component stylesheets are
//! scoped by default. The one substantive rule this type encodes is the
//! boundary-crossing rule: an isolated scope lets *inherited* and *custom*
//! properties cross its boundary and contains everything else. Custom
//! properties always cross because they always inherit; inherited properties
//! cross because inheritance flows through the tree regardless of scope; other
//! (non-inherited) properties are contained by an isolated scope.
//!
//! Scope-aware selector matching and module rewriting happen elsewhere (in the
//! matcher / `obscur-css`); this type only models scope identity and the
//! visibility of a property across the boundary.

use crate::property::{InheritedProperty, PropertyId};
use crate::scope::{ScopeId, ScopeKind};

/// A style scope, pairing its identity with its isolation behaviour (§4.15).
///
/// `Copy + Eq + Hash` so it can be stored and compared cheaply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StyleScope {
    id: ScopeId,
    kind: ScopeKind,
}

impl StyleScope {
    /// Creates a style scope with the given identity and behaviour.
    #[must_use]
    pub const fn new(id: ScopeId, kind: ScopeKind) -> Self {
        Self { id, kind }
    }

    /// Returns this scope's identifier.
    #[must_use]
    pub const fn id(self) -> ScopeId {
        self.id
    }

    /// Returns this scope's behaviour classification.
    #[must_use]
    pub const fn kind(self) -> ScopeKind {
        self.kind
    }

    /// Returns `true` when this scope isolates non-inherited properties at its
    /// boundary (delegates to [`ScopeKind::is_isolated`]).
    ///
    /// `Global` and `Open` scopes are not isolated; `Component` and `Module`
    /// scopes are (§4.15).
    #[must_use]
    pub const fn is_isolated(self) -> bool {
        self.kind.is_isolated()
    }

    /// Returns `true` when a property is of a kind that crosses a scope
    /// boundary — i.e. it inherits by default (§4.15).
    ///
    /// This is independent of this scope's kind: it classifies the *property*.
    /// Inherited properties cross because inheritance flows through the tree
    /// regardless of scope. Custom properties always inherit, so they always
    /// cross (they are represented among the inherited properties).
    /// Non-inherited properties do not cross and are contained by an isolated
    /// scope.
    #[must_use]
    pub fn property_crosses_boundary(property: PropertyId) -> bool {
        InheritedProperty::of(property).is_inherited()
    }

    /// Returns `true` when this scope *contains* the given property — i.e. the
    /// property does not escape the boundary.
    ///
    /// A `Global` or `Open` scope contains nothing: it performs no isolation, so
    /// every property is visible outside it. An isolated (`Component` /
    /// `Module`) scope contains exactly the properties that do not cross the
    /// boundary — every non-inherited, non-custom property (§4.15).
    #[must_use]
    pub fn contains_property(self, property: PropertyId) -> bool {
        self.is_isolated() && !Self::property_crosses_boundary(property)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component() -> StyleScope {
        StyleScope::new(ScopeId::from_raw(1), ScopeKind::Component)
    }

    #[test]
    fn accessors_round_trip() {
        let scope = component();
        assert_eq!(scope.id(), ScopeId::from_raw(1));
        assert_eq!(scope.kind(), ScopeKind::Component);
    }

    #[test]
    fn inherited_properties_cross_boundary() {
        assert!(StyleScope::property_crosses_boundary(PropertyId::Color));
        assert!(StyleScope::property_crosses_boundary(PropertyId::Visibility));
    }

    #[test]
    fn non_inherited_properties_do_not_cross() {
        assert!(!StyleScope::property_crosses_boundary(PropertyId::Opacity));
        assert!(!StyleScope::property_crosses_boundary(PropertyId::Width));
    }

    #[test]
    fn isolated_scope_contains_only_non_crossing() {
        let scope = component();
        // Non-inherited: contained.
        assert!(scope.contains_property(PropertyId::Opacity));
        // Inherited: crosses, not contained.
        assert!(!scope.contains_property(PropertyId::Color));
    }

    #[test]
    fn open_and_global_contain_nothing() {
        let open = StyleScope::new(ScopeId::from_raw(2), ScopeKind::Open);
        let global = StyleScope::new(ScopeId::GLOBAL, ScopeKind::Global);
        assert!(!open.contains_property(PropertyId::Opacity));
        assert!(!global.contains_property(PropertyId::Opacity));
    }
}
