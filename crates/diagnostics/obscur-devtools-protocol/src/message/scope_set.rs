use super::PermissionScope;

#[doc = include_str!("../../docs/api/message/permission_scope.doc/ScopeSet.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScopeSet(u8);

impl ScopeSet {
    /// Creates an empty scope set granting no capabilities.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Creates a scope set containing a single scope.
    #[must_use]
    pub const fn single(scope: PermissionScope) -> Self {
        Self(scope.bit())
    }

    /// A read-only scope set (`Read` only) — a safe default for production.
    #[must_use]
    pub const fn all_read() -> Self {
        Self::single(PermissionScope::Read)
    }

    /// An observer scope set granting `Read` + `Subscribe` (no control).
    #[must_use]
    pub const fn observer() -> Self {
        Self(PermissionScope::Read.bit() | PermissionScope::Subscribe.bit())
    }

    /// A scope set granting every scope — development builds only (spec §7.7).
    #[must_use]
    pub const fn all() -> Self {
        Self(
            PermissionScope::Read.bit()
                | PermissionScope::Subscribe.bit()
                | PermissionScope::Control.bit()
                | PermissionScope::Inject.bit(),
        )
    }

    /// Returns whether `scope` is present in the set.
    #[must_use]
    pub const fn contains(self, scope: PermissionScope) -> bool {
        self.0 & scope.bit() != 0
    }

    /// Returns a copy of the set with `scope` added.
    #[must_use]
    pub const fn with(self, scope: PermissionScope) -> Self {
        Self(self.0 | scope.bit())
    }

    #[doc = include_str!("../../docs/api/message/permission_scope.doc/without.md")]
    #[must_use]
    pub const fn without(self, scope: PermissionScope) -> Self {
        Self(self.0 & !scope.bit())
    }

    /// Returns whether the set grants no scopes.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
