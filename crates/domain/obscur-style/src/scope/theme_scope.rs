#![doc = include_str!("../../docs/api/scope/theme_scope.doc/_module.md")]

use crate::scope::ScopeId;

#[doc = include_str!("../../docs/api/scope/theme_scope.doc/ThemeScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeScope {
    scope: ScopeId,
    inherits_parent: bool,
}

impl ThemeScope {
    /// Creates a scoped theme, choosing whether it inherits from its parent.
    #[must_use]
    pub const fn new(scope: ScopeId, inherits_parent: bool) -> Self {
        Self {
            scope,
            inherits_parent,
        }
    }

    /// Creates a scoped theme rooted at `scope` that inherits from its parent —
    /// the default behaviour (§4.15).
    #[must_use]
    pub const fn rooted(scope: ScopeId) -> Self {
        Self::new(scope, true)
    }

    /// Returns the scope this theme is applied to.
    #[must_use]
    pub const fn scope(self) -> ScopeId {
        self.scope
    }

    #[doc = include_str!("../../docs/api/scope/theme_scope.doc/inherits_from_parent.md")]
    #[must_use]
    pub const fn inherits_from_parent(self) -> bool {
        self.inherits_parent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooted_inherits_by_default() {
        let theme = ThemeScope::rooted(ScopeId::from_raw(4));
        assert_eq!(theme.scope(), ScopeId::from_raw(4));
        assert!(theme.inherits_from_parent());
    }

    #[test]
    fn non_inheriting_theme() {
        let theme = ThemeScope::new(ScopeId::from_raw(4), false);
        assert!(!theme.inherits_from_parent());
    }
}
