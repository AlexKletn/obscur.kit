#![doc = include_str!("route_registry.doc/_module.md")]

use crate::route::{Route, RouteMatch, RoutePattern};

#[doc = include_str!("route_registry.doc/RouteRegistry.md")]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouteRegistry {
    #[doc = include_str!("route_registry.doc/routes.md")]
    routes: Vec<(RoutePattern, Box<str>)>,
}

impl RouteRegistry {
    #[doc = include_str!("route_registry.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("route_registry.doc/register.md")]
    pub fn register(&mut self, pattern: RoutePattern, target: impl Into<Box<str>>) {
        self.routes.push((pattern, target.into()));
    }

    #[doc = include_str!("route_registry.doc/with_route.md")]
    #[must_use]
    pub fn with_route(mut self, pattern: RoutePattern, target: impl Into<Box<str>>) -> Self {
        self.register(pattern, target);
        self
    }

    #[doc = include_str!("route_registry.doc/resolve.md")]
    #[must_use]
    pub fn resolve(&self, route: &Route) -> Option<(RouteMatch, &str)> {
        self.routes.iter().find_map(|(pattern, target)| {
            RouteMatch::try_match(route, pattern).map(|matched| (matched, &**target))
        })
    }

    #[doc = include_str!("route_registry.doc/len.md")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    #[doc = include_str!("route_registry.doc/is_empty.md")]
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
        let (_, target) = registry
            .resolve(&Route::parse("/users/1"))
            .expect("resolves");
        assert_eq!(target, "specific");
        // A path only the wildcard matches falls through to the catch-all.
        let (_, target) = registry
            .resolve(&Route::parse("/other/x"))
            .expect("resolves");
        assert_eq!(target, "catch-all");
    }

    #[test]
    fn resolve_no_match_is_none() {
        let registry = RouteRegistry::new().with_route(RoutePattern::parse("/users/:id"), "user");
        assert!(registry.resolve(&Route::parse("/posts/1")).is_none());
    }
}
