#![doc = include_str!("style_scope.doc/_module.md")]

use crate::property::{InheritedProperty, PropertyId};
use crate::scope::{ScopeId, ScopeKind};

#[doc = include_str!("style_scope.doc/StyleScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StyleScope {
    id: ScopeId,
    kind: ScopeKind,
}

impl StyleScope {
    #[doc = include_str!("style_scope.doc/new.md")]
    #[must_use]
    pub const fn new(id: ScopeId, kind: ScopeKind) -> Self {
        Self { id, kind }
    }

    #[doc = include_str!("style_scope.doc/id.md")]
    #[must_use]
    pub const fn id(self) -> ScopeId {
        self.id
    }

    #[doc = include_str!("style_scope.doc/kind.md")]
    #[must_use]
    pub const fn kind(self) -> ScopeKind {
        self.kind
    }

    #[doc = include_str!("style_scope.doc/is_isolated.md")]
    #[must_use]
    pub const fn is_isolated(self) -> bool {
        self.kind.is_isolated()
    }

    #[doc = include_str!("style_scope.doc/property_crosses_boundary.md")]
    #[must_use]
    pub fn property_crosses_boundary(property: PropertyId) -> bool {
        InheritedProperty::of(property).is_inherited()
    }

    #[doc = include_str!("style_scope.doc/contains_property.md")]
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
        assert!(StyleScope::property_crosses_boundary(
            PropertyId::Visibility
        ));
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
