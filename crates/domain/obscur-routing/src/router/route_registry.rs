//! The registered pattern-to-target mapping for a router (spec §6.13).

use crate::route::{Route, RouteMatch, RoutePattern};

/// The ordered pattern-to-target mapping a router resolves against (spec §6.13
/// route-to-screen/component mapping, deep-link mapping).
///
/// Each registered entry pairs a [`RoutePattern`] with an application-chosen
/// **target id** — a screen or component name the toolkit stores verbatim and
/// never interprets. Resolution walks the registrations in order and returns the
/// first pattern that matches, so **registration order is priority**: register
/// more specific patterns before broader (for example wildcard) ones.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteRegistry {
    /// The registered pattern/target pairs, in priority (registration) order.
    routes: Vec<(RoutePattern, Box<str>)>,
}

impl RouteRegistry {
    /// Creates an empty registry with no registered routes.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `pattern` mapped to the application-chosen `target` id.
    ///
    /// Registrations are appended, so earlier registrations take priority during
    /// [`resolve`](RouteRegistry::resolve).
    pub fn register(&mut self, pattern: RoutePattern, target: impl Into<Box<str>>) {
        self.routes.push((pattern, target.into()));
    }

    /// Registers `pattern` mapped to `target` and returns `self` (builder style).
    #[must_use]
    pub fn with_route(mut self, pattern: RoutePattern, target: impl Into<Box<str>>) -> Self {
        self.register(pattern, target);
        self
    }

    /// Resolves `route` to the first matching registration (spec §6.13).
    ///
    /// Returns the [`RouteMatch`] (captured params and matched path) paired with
    /// the target id of the first registered pattern that matches `route`, or
    /// `None` when no pattern matches (a deep-link miss). Registration order is
    /// priority: the first match wins.
    #[must_use]
    pub fn resolve(&self, route: &Route) -> Option<(RouteMatch, &str)> {
        self.routes.iter().find_map(|(pattern, target)| {
            RouteMatch::try_match(route, pattern).map(|matched| (matched, &**target))
        })
    }

    /// Returns the number of registered routes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Returns whether no routes are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_len() {
        let mut registry = RouteRegistry::new();
        assert!(registry.is_empty());
        registry.register(RoutePattern::parse("/users/:id"), "user-screen");
        assert_eq!(registry.len(), 1);
        assert!(!registry.is_empty());
    }

    #[test]
    fn resolve_returns_match_and_target() {
        let registry = RouteRegistry::new().with_route(RoutePattern::parse("/users/:id"), "user");
        let route = Route::parse("/users/42?tab=posts");
        let (matched, target) = registry.resolve(&route).expect("should resolve");
        assert_eq!(target, "user");
        assert_eq!(matched.matched_path(), "/users/42");
        assert_eq!(matched.params().get("id"), Some("42"));
    }

    #[test]
    fn resolve_first_match_wins() {
        // A specific pattern registered before a catch-all wildcard.
        let registry = RouteRegistry::new()
            .with_route(RoutePattern::parse("/users/:id"), "specific")
            .with_route(RoutePattern::parse("/*rest"), "catch-all");
        let (_, target) = registry.resolve(&Route::parse("/users/1")).expect("resolves");
        assert_eq!(target, "specific");
        // A path only the wildcard matches falls through to the catch-all.
        let (_, target) = registry.resolve(&Route::parse("/other/x")).expect("resolves");
        assert_eq!(target, "catch-all");
    }

    #[test]
    fn resolve_no_match_is_none() {
        let registry = RouteRegistry::new().with_route(RoutePattern::parse("/users/:id"), "user");
        assert!(registry.resolve(&Route::parse("/posts/1")).is_none());
    }
}
