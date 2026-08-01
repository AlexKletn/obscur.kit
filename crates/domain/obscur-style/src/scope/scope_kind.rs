#![doc = include_str!("scope_kind.doc/_module.md")]

#[doc = include_str!("scope_kind.doc/ScopeKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    #[doc = include_str!("scope_kind.doc/Global.md")]
    Global,
    #[doc = include_str!("scope_kind.doc/Component.md")]
    Component,
    #[doc = include_str!("scope_kind.doc/Module.md")]
    Module,
    #[doc = include_str!("scope_kind.doc/Open.md")]
    Open,
}

impl ScopeKind {
    #[doc = include_str!("scope_kind.doc/is_isolated.md")]
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
