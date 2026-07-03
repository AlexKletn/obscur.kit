//! The result of matching a [`Route`] against a [`RoutePattern`] (spec §6.13).

use crate::route::{Route, RouteParams, RoutePattern};

/// The outcome of successfully matching a [`Route`] against a [`RoutePattern`]
/// (spec §6.13).
///
/// A `RouteMatch` bundles the [`RouteParams`] captured by the pattern with the
/// concrete path that matched, giving a caller everything needed to resolve the
/// route to a screen or component without re-running the match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteMatch {
    /// The parameters captured by the pattern.
    params: RouteParams,
    /// The path that was matched.
    matched_path: Box<str>,
}

impl RouteMatch {
    /// Creates a match from captured `params` and the `matched_path`.
    #[must_use]
    pub fn new(params: RouteParams, matched_path: impl Into<Box<str>>) -> Self {
        Self {
            params,
            matched_path: matched_path.into(),
        }
    }

    /// Attempts to match `route`'s path against `pattern` (spec §6.13).
    ///
    /// Returns `Some(RouteMatch)` with the captured parameters and the route's
    /// path when the pattern matches, or `None` otherwise. Only the path
    /// participates in matching; the route's query and fragment are ignored.
    #[must_use]
    pub fn try_match(route: &Route, pattern: &RoutePattern) -> Option<Self> {
        pattern
            .match_path(route.path())
            .map(|params| Self::new(params, route.path()))
    }

    /// Returns the captured parameters.
    #[must_use]
    pub fn params(&self) -> &RouteParams {
        &self.params
    }

    /// Returns the path that matched.
    #[must_use]
    pub fn matched_path(&self) -> &str {
        &self.matched_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_match_success() {
        let route = Route::parse("/users/42?tab=posts");
        let pattern = RoutePattern::parse("/users/:id");
        let matched = RouteMatch::try_match(&route, &pattern).expect("should match");
        assert_eq!(matched.matched_path(), "/users/42");
        assert_eq!(matched.params().get("id"), Some("42"));
    }

    #[test]
    fn try_match_non_match() {
        let route = Route::parse("/posts/42");
        let pattern = RoutePattern::parse("/users/:id");
        assert!(RouteMatch::try_match(&route, &pattern).is_none());
    }

    #[test]
    fn new_constructs_directly() {
        let mut params = RouteParams::new();
        params.insert("id", "7");
        let matched = RouteMatch::new(params, "/users/7");
        assert_eq!(matched.matched_path(), "/users/7");
        assert_eq!(matched.params().get("id"), Some("7"));
    }
}
