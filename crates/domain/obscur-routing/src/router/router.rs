#![doc = include_str!("router.doc/_module.md")]

use crate::history::{HistoryProvider, MemoryHistory, NavigationEntry};
use crate::route::{Route, RouteState};
use crate::router::{NavigationResult, RouteRegistry};

#[doc = include_str!("router.doc/Router.md")]
#[derive(Debug)]
pub struct Router {
    #[doc = include_str!("router.doc/registry.md")]
    registry: RouteRegistry,
    #[doc = include_str!("router.doc/history.md")]
    history: Box<dyn HistoryProvider>,
}

impl Router {
    #[doc = include_str!("router.doc/new.md")]
    #[must_use]
    pub fn new(registry: RouteRegistry) -> Self {
        Self {
            registry,
            history: Box::new(MemoryHistory::new()),
        }
    }

    #[doc = include_str!("router.doc/with_history.md")]
    #[must_use]
    pub fn with_history(registry: RouteRegistry, history: Box<dyn HistoryProvider>) -> Self {
        Self { registry, history }
    }

    #[doc = include_str!("router.doc/navigate.md")]
    pub fn navigate(&mut self, path: &str) -> NavigationResult {
        self.navigate_with_state(path, RouteState::empty())
    }

    #[doc = include_str!("router.doc/navigate_with_state.md")]
    pub fn navigate_with_state(&mut self, path: &str, state: RouteState) -> NavigationResult {
        let route = Route::parse(path);
        match self.registry.resolve(&route) {
            Some((matched, target)) => {
                let result = NavigationResult::matched(
                    Box::<str>::from(target),
                    matched.params().clone(),
                    route.path(),
                );
                self.history.push(route, state);
                result
            }
            None => NavigationResult::not_found(route.path()),
        }
    }

    #[doc = include_str!("router.doc/back.md")]
    pub fn back(&mut self) -> Option<NavigationResult> {
        // Move the cursor; the borrowed entry is dropped at the semicolon so
        // resolve_current can re-borrow self.
        self.history.back()?;
        self.resolve_current()
    }

    #[doc = include_str!("router.doc/forward.md")]
    pub fn forward(&mut self) -> Option<NavigationResult> {
        // Move the cursor; the borrowed entry is dropped at the semicolon so
        // resolve_current can re-borrow self.
        self.history.forward()?;
        self.resolve_current()
    }

    #[doc = include_str!("router.doc/resolve_current.md")]
    fn resolve_current(&self) -> Option<NavigationResult> {
        let entry = self.history.current()?;
        let route = entry.route();
        Some(match self.registry.resolve(route) {
            Some((matched, target)) => NavigationResult::matched(
                Box::<str>::from(target),
                matched.params().clone(),
                route.path(),
            ),
            None => NavigationResult::not_found(route.path()),
        })
    }

    #[doc = include_str!("router.doc/current.md")]
    #[must_use]
    pub fn current(&self) -> Option<&NavigationEntry> {
        self.history.current()
    }

    #[doc = include_str!("router.doc/can_go_back.md")]
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        self.history.can_go_back()
    }

    #[doc = include_str!("router.doc/can_go_forward.md")]
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        self.history.can_go_forward()
    }

    #[doc = include_str!("router.doc/registry_2.md")]
    #[must_use]
    pub fn registry(&self) -> &RouteRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::route::RoutePattern;

    fn user_registry() -> RouteRegistry {
        RouteRegistry::new()
            .with_route(RoutePattern::parse("/users/:id"), "user")
            .with_route(RoutePattern::parse("/about"), "about")
    }

    #[test]
    fn navigate_matched_pushes_history() {
        let mut router = Router::new(user_registry());
        let result = router.navigate("/users/42?tab=posts");
        assert!(result.is_matched());
        assert_eq!(result.target(), Some("user"));
        assert_eq!(result.params().and_then(|p| p.get("id")), Some("42"));
        assert_eq!(result.path(), "/users/42");
        // History was pushed.
        assert_eq!(router.current().unwrap().route().path(), "/users/42");
        assert!(!router.can_go_back());
    }

    #[test]
    fn navigate_not_found_does_not_push_history() {
        let mut router = Router::new(user_registry());
        let result = router.navigate("/missing/page");
        assert!(!result.is_matched());
        assert_eq!(result.path(), "/missing/page");
        // History untouched: no current entry.
        assert!(router.current().is_none());
    }

    #[test]
    fn navigate_with_state_carries_state() {
        let mut router = Router::new(user_registry());
        let result = router.navigate_with_state("/about", RouteState::new("scroll", "{\"y\":5}"));
        assert!(result.is_matched());
        let entry = router.current().unwrap();
        assert_eq!(entry.state().kind(), "scroll");
        assert_eq!(entry.state().payload(), "{\"y\":5}");
    }

    #[test]
    fn back_and_forward_re_resolve() {
        let mut router = Router::new(user_registry());
        router.navigate("/users/1");
        router.navigate("/about");

        let back = router.back().expect("can go back");
        assert!(back.is_matched());
        assert_eq!(back.target(), Some("user"));
        assert_eq!(back.params().and_then(|p| p.get("id")), Some("1"));
        assert_eq!(router.current().unwrap().route().path(), "/users/1");

        let forward = router.forward().expect("can go forward");
        assert!(forward.is_matched());
        assert_eq!(forward.target(), Some("about"));
        assert_eq!(router.current().unwrap().route().path(), "/about");
    }

    #[test]
    fn back_and_forward_at_bounds_return_none() {
        let mut router = Router::new(user_registry());
        router.navigate("/about");
        // At the only entry: neither move is possible.
        assert!(router.back().is_none());
        assert!(router.forward().is_none());
    }

    #[test]
    fn back_or_forward_to_unregistered_pattern_yields_not_found() {
        // Pre-populate history with a route the registry does NOT register, so
        // re-resolution on forward misses.
        let mut history = MemoryHistory::new();
        history.push(Route::parse("/registered"), RouteState::empty());
        history.push(Route::parse("/orphan"), RouteState::empty());
        let mut router = Router::with_history(
            RouteRegistry::new().with_route(RoutePattern::parse("/registered"), "reg"),
            Box::new(history),
        );

        // current is /orphan; going back lands on the registered route.
        let back = router.back().expect("can go back");
        assert!(back.is_matched());
        assert_eq!(back.target(), Some("reg"));

        // Going forward returns to /orphan, whose pattern is unregistered.
        let forward = router.forward().expect("can go forward");
        assert!(!forward.is_matched());
        assert_eq!(forward.path(), "/orphan");
    }

    #[test]
    fn registry_accessor_exposes_registrations() {
        let router = Router::new(user_registry());
        assert_eq!(router.registry().len(), 2);
    }
}
