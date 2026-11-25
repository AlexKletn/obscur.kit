#![doc = include_str!("../../docs/api/scope/scope_kind.doc/_module.md")]

#[doc = include_str!("../../docs/api/scope/scope_kind.doc/ScopeKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    /// The global application scope: no isolation. Global stylesheets live here
    /// and everything is visible (§4.15).
    Global,
    /// A component scope: scoped by default, so non-inherited properties are
    /// contained at the boundary while inherited and custom properties still
    /// cross (§4.15).
    Component,
    /// A component scope in CSS-module mode: like [`ScopeKind::Component`] but
    /// additionally isolating its local identifiers so internal class names need
    /// not be globally unique (§4.15).
    Module,
    /// A component scope that is explicitly opened / made global despite being a
    /// component: it performs no isolation, matching [`ScopeKind::Global`]'s
    /// visibility (§4.15).
    Open,
}

impl ScopeKind {
    #[doc = include_str!("../../docs/api/scope/scope_kind.doc/is_isolated.md")]
    #[must_use]
    pub const fn is_isolated(self) -> bool {
        matches!(self, Self::Component | Self::Module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolation_matches_kind() {
        assert!(ScopeKind::Component.is_isolated());
        assert!(ScopeKind::Module.is_isolated());
        assert!(!ScopeKind::Global.is_isolated());
        assert!(!ScopeKind::Open.is_isolated());
    }

    #[test]
    fn is_hashable() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ScopeKind::Component);
        set.insert(ScopeKind::Component);
        set.insert(ScopeKind::Global);
        assert_eq!(set.len(), 2);
    }
}
