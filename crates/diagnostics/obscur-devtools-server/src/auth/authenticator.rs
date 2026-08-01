#![doc = include_str!("authenticator.doc/_module.md")]

use std::collections::HashMap;

use obscur_devtools_protocol::message::{PermissionScope, ScopeSet};

#[doc = include_str!("authenticator.doc/Authenticator.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Authenticator {
    tokens: HashMap<Box<str>, ScopeSet>,
    exclude_control: bool,
    exclude_inject: bool,
}

impl Authenticator {
    #[doc = include_str!("authenticator.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            exclude_control: false,
            exclude_inject: false,
        }
    }

    #[doc = include_str!("authenticator.doc/production.md")]
    #[must_use]
    pub fn production() -> Self {
        Self {
            tokens: HashMap::new(),
            exclude_control: true,
            exclude_inject: true,
        }
    }

    #[doc = include_str!("authenticator.doc/with_control_excluded.md")]
    #[must_use]
    pub fn with_control_excluded(mut self) -> Self {
        self.exclude_control = true;
        self
    }

    #[doc = include_str!("authenticator.doc/with_inject_excluded.md")]
    #[must_use]
    pub fn with_inject_excluded(mut self) -> Self {
        self.exclude_inject = true;
        self
    }

    #[doc = include_str!("authenticator.doc/register_token.md")]
    pub fn register_token(&mut self, token: impl Into<Box<str>>, scopes: ScopeSet) {
        self.tokens.insert(token.into(), scopes);
    }

    #[doc = include_str!("authenticator.doc/authenticate.md")]
    #[must_use]
    pub fn authenticate(&self, token: &str) -> Option<ScopeSet> {
        let granted = *self.tokens.get(token)?;
        Some(self.mask(granted))
    }

    #[doc = include_str!("authenticator.doc/is_enabled.md")]
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        !self.tokens.is_empty()
    }

    #[doc = include_str!("authenticator.doc/excludes_control.md")]
    #[must_use]
    pub fn excludes_control(&self) -> bool {
        self.exclude_control
    }

    #[doc = include_str!("authenticator.doc/mask.md")]
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
