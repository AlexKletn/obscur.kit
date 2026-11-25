//! Capability scopes and the granted scope set (spec §7.7).

#[doc = include_str!("../../docs/api/message/permission_scope.doc/PermissionScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionScope {
    /// Read live inspectable state (tree, styles, layout, …).
    Read,
    /// Subscribe to and unsubscribe from event streams.
    Subscribe,
    /// Issue control actions (pause, step, inspect-mode).
    Control,
    /// Inject synthetic input/command events.
    Inject,
}

impl PermissionScope {
    /// The bit this scope occupies within a [`ScopeSet`](super::ScopeSet).
    pub(super) const fn bit(self) -> u8 {
        match self {
            Self::Read => 1 << 0,
            Self::Subscribe => 1 << 1,
            Self::Control => 1 << 2,
            Self::Inject => 1 << 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::ScopeSet;
    use super::*;

    #[test]
    fn empty_contains_nothing() {
        let s = ScopeSet::empty();
        assert!(s.is_empty());
        assert!(!s.contains(PermissionScope::Read));
        assert_eq!(s, ScopeSet::default());
    }

    #[test]
    fn all_read_is_read_only() {
        let s = ScopeSet::all_read();
        assert!(s.contains(PermissionScope::Read));
        assert!(!s.contains(PermissionScope::Subscribe));
        assert!(!s.contains(PermissionScope::Control));
        assert!(!s.contains(PermissionScope::Inject));
    }

    #[test]
    fn observer_grants_read_and_subscribe_only() {
        let s = ScopeSet::observer();
        assert!(s.contains(PermissionScope::Read));
        assert!(s.contains(PermissionScope::Subscribe));
        assert!(!s.contains(PermissionScope::Control));
        assert!(!s.contains(PermissionScope::Inject));
    }

    #[test]
    fn all_grants_every_scope() {
        let s = ScopeSet::all();
        for scope in [
            PermissionScope::Read,
            PermissionScope::Subscribe,
            PermissionScope::Control,
            PermissionScope::Inject,
        ] {
            assert!(s.contains(scope));
        }
        assert!(!s.is_empty());
    }

    #[test]
    fn with_and_without_are_inverse() {
        let s = ScopeSet::empty().with(PermissionScope::Control);
        assert!(s.contains(PermissionScope::Control));
        let s = s.without(PermissionScope::Control);
        assert!(!s.contains(PermissionScope::Control));
    }

    #[test]
    fn without_excludes_control_from_full_set() {
        let production = ScopeSet::all().without(PermissionScope::Control);
        assert!(!production.contains(PermissionScope::Control));
        assert!(production.contains(PermissionScope::Read));
    }
}
