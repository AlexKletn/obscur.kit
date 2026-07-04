//! Token → scope authenticator for DevTools sessions (spec §7.7).

use std::collections::HashMap;

use obscur_devtools_protocol::message::{PermissionScope, ScopeSet};

/// A token → [`ScopeSet`] registry that authenticates DevTools sessions.
///
/// The endpoint is **disabled by default** (spec §7.7): a freshly
/// [`new`](Self::new)'d authenticator has an empty registry, so
/// [`authenticate`](Self::authenticate) rejects every token until the host
/// explicitly [`register_token`](Self::register_token)s one. Tokens are opaque
/// local capabilities and are never logged.
///
/// A [`production`](Self::production) authenticator additionally **strips the
/// `Control` (and `Inject`) scopes** from every granted set, so a token that
/// would grant full access in development can only ever observe in production —
/// the dangerous capabilities are removed regardless of what was registered.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Authenticator {
    tokens: HashMap<Box<str>, ScopeSet>,
    exclude_control: bool,
    exclude_inject: bool,
}

impl Authenticator {
    /// Creates an empty authenticator that rejects every token.
    ///
    /// This is the disabled-by-default state (spec §7.7); the host must
    /// [`register_token`](Self::register_token) before any session can connect.
    #[must_use]
    pub fn new() -> Self {
        Self { tokens: HashMap::new(), exclude_control: false, exclude_inject: false }
    }

    /// Creates a production authenticator that strips the `Control` and
    /// `Inject` scopes from every granted set (spec §7.7).
    ///
    /// Registered tokens can still grant `Read`/`Subscribe`, but the two
    /// dangerous scopes are removed unconditionally, so production builds can
    /// never steer or inject into the runtime.
    #[must_use]
    pub fn production() -> Self {
        Self { tokens: HashMap::new(), exclude_control: true, exclude_inject: true }
    }

    /// Returns a copy of this authenticator that excludes the `Control` scope
    /// from every granted set (spec §7.7).
    #[must_use]
    pub fn with_control_excluded(mut self) -> Self {
        self.exclude_control = true;
        self
    }

    /// Returns a copy of this authenticator that also excludes the `Inject`
    /// scope from every granted set (spec §7.7).
    #[must_use]
    pub fn with_inject_excluded(mut self) -> Self {
        self.exclude_inject = true;
        self
    }

    /// Registers `token` as granting `scopes`.
    ///
    /// Re-registering the same token replaces its scopes. The stored scopes are
    /// the *requested* grant; any production exclusions are applied at
    /// [`authenticate`](Self::authenticate) time.
    pub fn register_token(&mut self, token: impl Into<Box<str>>, scopes: ScopeSet) {
        self.tokens.insert(token.into(), scopes);
    }

    /// Authenticates `token`, returning the scopes granted to it.
    ///
    /// Returns [`None`] for an unknown token (including any token when the
    /// registry is empty). For a known token the registered scopes are returned
    /// with any production exclusions (`Control`/`Inject`) removed.
    #[must_use]
    pub fn authenticate(&self, token: &str) -> Option<ScopeSet> {
        let granted = *self.tokens.get(token)?;
        Some(self.mask(granted))
    }

    /// Returns whether any token has been registered.
    ///
    /// An authenticator with no tokens is effectively disabled (spec §7.7).
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        !self.tokens.is_empty()
    }

    /// Returns whether the `Control` scope is excluded from every grant.
    #[must_use]
    pub fn excludes_control(&self) -> bool {
        self.exclude_control
    }

    /// Applies the configured production exclusions to a granted scope set.
    fn mask(&self, mut scopes: ScopeSet) -> ScopeSet {
        if self.exclude_control {
            scopes = scopes.without(PermissionScope::Control);
        }
        if self.exclude_inject {
            scopes = scopes.without(PermissionScope::Inject);
        }
        scopes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_rejects_every_token() {
        let auth = Authenticator::new();
        assert!(!auth.is_enabled());
        assert_eq!(auth.authenticate("anything"), None);
    }

    #[test]
    fn registered_token_returns_scopes() {
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::observer());
        assert!(auth.is_enabled());
        assert_eq!(auth.authenticate("dev"), Some(ScopeSet::observer()));
    }

    #[test]
    fn unknown_token_is_none() {
        let mut auth = Authenticator::new();
        auth.register_token("dev", ScopeSet::all());
        assert_eq!(auth.authenticate("other"), None);
    }

    #[test]
    fn production_strips_control_and_inject() {
        let mut auth = Authenticator::production();
        auth.register_token("dev", ScopeSet::all());
        let granted = auth.authenticate("dev").expect("known token");
        assert!(granted.contains(PermissionScope::Read));
        assert!(granted.contains(PermissionScope::Subscribe));
        assert!(!granted.contains(PermissionScope::Control));
        assert!(!granted.contains(PermissionScope::Inject));
    }

    #[test]
    fn with_control_excluded_strips_only_control() {
        let mut auth = Authenticator::new().with_control_excluded();
        auth.register_token("dev", ScopeSet::all());
        let granted = auth.authenticate("dev").expect("known token");
        assert!(!granted.contains(PermissionScope::Control));
        assert!(granted.contains(PermissionScope::Inject));
        assert!(auth.excludes_control());
    }
}
