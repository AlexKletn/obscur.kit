#![doc = include_str!("permission_scope.doc/_module.md")]

#[doc = include_str!("permission_scope.doc/PermissionScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionScope {
    #[doc = include_str!("permission_scope.doc/Read.md")]
    Read,
    #[doc = include_str!("permission_scope.doc/Subscribe.md")]
    Subscribe,
    #[doc = include_str!("permission_scope.doc/Control.md")]
    Control,
    #[doc = include_str!("permission_scope.doc/Inject.md")]
    Inject,
}

impl PermissionScope {
    #[doc = include_str!("permission_scope.doc/bit.md")]
    const fn bit(self) -> u8 {
        match self {
            Self::Read => 1 << 0,
            Self::Subscribe => 1 << 1,
            Self::Control => 1 << 2,
            Self::Inject => 1 << 3,
        }
    }
}

#[doc = include_str!("permission_scope.doc/ScopeSet.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScopeSet(u8);

impl ScopeSet {
    #[doc = include_str!("permission_scope.doc/empty.md")]
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[doc = include_str!("permission_scope.doc/single.md")]
    #[must_use]
    pub const fn single(scope: PermissionScope) -> Self {
        Self(scope.bit())
    }

    #[doc = include_str!("permission_scope.doc/all_read.md")]
    #[must_use]
    pub const fn all_read() -> Self {
        Self::single(PermissionScope::Read)
    }

    #[doc = include_str!("permission_scope.doc/observer.md")]
    #[must_use]
    pub const fn observer() -> Self {
        Self(PermissionScope::Read.bit() | PermissionScope::Subscribe.bit())
    }

    #[doc = include_str!("permission_scope.doc/all.md")]
    #[must_use]
    pub const fn all() -> Self {
        Self(
            PermissionScope::Read.bit()
                | PermissionScope::Subscribe.bit()
                | PermissionScope::Control.bit()
                | PermissionScope::Inject.bit(),
        )
    }

    #[doc = include_str!("permission_scope.doc/contains.md")]
    #[must_use]
    pub const fn contains(self, scope: PermissionScope) -> bool {
        self.0 & scope.bit() != 0
    }

    #[doc = include_str!("permission_scope.doc/with.md")]
    #[must_use]
    pub const fn with(self, scope: PermissionScope) -> Self {
        Self(self.0 | scope.bit())
    }

    #[doc = include_str!("permission_scope.doc/without.md")]
    #[must_use]
    pub const fn without(self, scope: PermissionScope) -> Self {
        Self(self.0 & !scope.bit())
    }

    #[doc = include_str!("permission_scope.doc/is_empty.md")]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[cfg(test)]
mod tests {
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
