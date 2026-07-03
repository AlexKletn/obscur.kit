//! The typed outcome of a router navigation (spec §6.13).

use crate::route::RouteParams;

/// The typed outcome of a navigation through the router (spec §6.13).
///
/// A navigation either resolves a path to a registered target
/// ([`Matched`](NavigationResult::Matched), carrying the target id, the params
/// the pattern captured, and the matched path) or finds no registered pattern
/// ([`NotFound`](NavigationResult::NotFound), a deep-link miss carrying the path
/// that failed). The [`is_matched`](NavigationResult::is_matched),
/// [`target`](NavigationResult::target), and
/// [`params`](NavigationResult::params) helpers inspect the outcome without
/// matching on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationResult {
    /// A registered pattern matched: the path resolved to `target` with the
    /// captured `params`.
    Matched {
        /// The application-chosen target id the pattern was registered under.
        target: Box<str>,
        /// The parameters captured from the matched path.
        params: RouteParams,
        /// The path that matched.
        path: Box<str>,
    },
    /// No registered pattern matched the path (a deep-link miss).
    NotFound {
        /// The path that failed to resolve.
        path: Box<str>,
    },
}

impl NavigationResult {
    /// Creates a [`Matched`](NavigationResult::Matched) result.
    #[must_use]
    pub fn matched(
        target: impl Into<Box<str>>,
        params: RouteParams,
        path: impl Into<Box<str>>,
    ) -> Self {
        Self::Matched {
            target: target.into(),
            params,
            path: path.into(),
        }
    }

    /// Creates a [`NotFound`](NavigationResult::NotFound) result for `path`.
    #[must_use]
    pub fn not_found(path: impl Into<Box<str>>) -> Self {
        Self::NotFound { path: path.into() }
    }

    /// Returns whether this is a [`Matched`](NavigationResult::Matched) outcome.
    #[must_use]
    pub const fn is_matched(&self) -> bool {
        matches!(self, Self::Matched { .. })
    }

    /// Returns the resolved target id, or `None` for a
    /// [`NotFound`](NavigationResult::NotFound) outcome.
    #[must_use]
    pub fn target(&self) -> Option<&str> {
        match self {
            Self::Matched { target, .. } => Some(target),
            Self::NotFound { .. } => None,
        }
    }

    /// Returns the captured params, or `None` for a
    /// [`NotFound`](NavigationResult::NotFound) outcome.
    #[must_use]
    pub const fn params(&self) -> Option<&RouteParams> {
        match self {
            Self::Matched { params, .. } => Some(params),
            Self::NotFound { .. } => None,
        }
    }

    /// Returns the path this result concerns (matched or not found).
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::Matched { path, .. } | Self::NotFound { path } => path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matched_helpers() {
        let mut params = RouteParams::new();
        params.insert("id", "42");
        let result = NavigationResult::matched("user", params, "/users/42");
        assert!(result.is_matched());
        assert_eq!(result.target(), Some("user"));
        assert_eq!(result.params().and_then(|p| p.get("id")), Some("42"));
        assert_eq!(result.path(), "/users/42");
    }

    #[test]
    fn not_found_helpers() {
        let result = NavigationResult::not_found("/missing");
        assert!(!result.is_matched());
        assert_eq!(result.target(), None);
        assert!(result.params().is_none());
        assert_eq!(result.path(), "/missing");
    }
}
