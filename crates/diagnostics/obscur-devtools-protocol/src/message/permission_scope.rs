//! Capability scopes and the granted scope set (spec §7.7).

/// A single class of DevTools capability an endpoint may hold.
///
/// Scopes separate the four capability classes so the server can grant them
/// independently: reading live state, subscribing to event streams, issuing
/// control actions, and injecting synthetic events (spec §7.7).
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
    /// The bit this scope occupies within a [`ScopeSet`].
    const fn bit(self) -> u8 {
        match self {
            Self::Read => 1 << 0,
            Self::Subscribe => 1 << 1,
            Self::Control => 1 << 2,
            Self::Inject => 1 << 3,
        }
    }
}

/// The set of scopes granted to an endpoint.
///
/// Modeled as a small bitset over the four [`PermissionScope`] values. The
/// server builds this from its auth store during the handshake and the runtime
/// consults it before executing each command.
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

    /// Returns a copy of the set with `scope` removed.
    ///
    /// Used to exclude control (and injection) entirely in production builds
    /// (spec §7.7).
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
