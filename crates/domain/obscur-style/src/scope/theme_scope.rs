//! A scoped theme applied to a subtree.
//!
//! Per §4.14/§4.15, a theme can be scoped to a subtree rooted at a style scope.
//! A scoped theme inherits from its root / parent theme by default, so it
//! *supplements* the ambient theme (adding or overriding tokens) rather than
//! replacing it; a non-inheriting scoped theme instead starts fresh from the
//! root theme. This type models that identity plus the inherit-from-parent flag;
//! resolving the actual theme tokens lives elsewhere.

use crate::scope::ScopeId;

/// A theme scoped to the subtree rooted at a style scope (§4.14/§4.15).
///
/// `Copy + Eq + Hash`.
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

    /// Returns `true` when this scoped theme inherits from its parent theme.
    ///
    /// When it does, the scoped theme supplements the ambient theme rather than
    /// replacing it; when it does not, it starts fresh from the root theme
    /// (§4.14/§4.15).
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
