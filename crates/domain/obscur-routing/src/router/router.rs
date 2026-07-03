//! The router composing a route registry with a history provider (spec §6.13).

use crate::history::{HistoryProvider, MemoryHistory, NavigationEntry};
use crate::route::{Route, RouteState};
use crate::router::{NavigationResult, RouteRegistry};

/// The router: a [`RouteRegistry`] composed with a history provider (spec
/// §6.13).
///
/// The router is the toolkit's composition point for URL-like navigation. It
/// owns the pattern-to-target [`RouteRegistry`] and a boxed
/// [`HistoryProvider`], and turns a path into a typed [`NavigationResult`] while
/// maintaining the back/forward history stack. Application navigation *policy*
/// (guards, redirects, what a target means) stays outside the toolkit.
///
/// `Router` derives only [`Debug`] — not `Clone` or `PartialEq` — because it
/// holds a `Box<dyn HistoryProvider>`, a trait object that is neither `Clone`
/// nor `PartialEq`. The [`HistoryProvider`] trait carries a
/// [`std::fmt::Debug`] supertrait so the boxed provider is `Debug`.
#[derive(Debug)]
pub struct Router {
    /// The pattern-to-target mapping navigations resolve against.
    registry: RouteRegistry,
    /// The replaceable history stack backing back/forward navigation.
    history: Box<dyn HistoryProvider>,
}

impl Router {
    /// Creates a router for `registry`, backed by an in-memory history
    /// ([`MemoryHistory`]).
    #[must_use]
    pub fn new(registry: RouteRegistry) -> Self {
        Self {
            registry,
            history: Box::new(MemoryHistory::new()),
        }
    }

    /// Creates a router for `registry` backed by the supplied `history`
    /// provider.
    #[must_use]
    pub fn with_history(registry: RouteRegistry, history: Box<dyn HistoryProvider>) -> Self {
        Self { registry, history }
    }

    /// Navigates to `path` with empty state, returning the typed outcome (spec
    /// §6.13).
    ///
    /// See [`navigate_with_state`](Router::navigate_with_state) for the full
    /// contract; this is the empty-state shorthand.
    pub fn navigate(&mut self, path: &str) -> NavigationResult {
        self.navigate_with_state(path, RouteState::empty())
    }

    /// Navigates to `path`, attaching `state` on a match (spec §6.13).
    ///
    /// The `path` is parsed into a [`Route`] and resolved against the registry.
    /// On a match the route is pushed onto history (carrying `state`) and a
    /// [`NavigationResult::Matched`] is returned. On no match a
    /// [`NavigationResult::NotFound`] is returned and **history is not mutated** —
    /// a failed navigation leaves the back/forward stack untouched.
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

    /// Moves history back one entry and re-resolves it, or returns `None` when
    /// already at the start (spec §6.13).
    ///
    /// The new current entry's route is re-resolved against the *current*
    /// registry, so an entry whose pattern is no longer registered yields a
    /// [`NavigationResult::NotFound`] rather than a stale match.
    pub fn back(&mut self) -> Option<NavigationResult> {
        // Move the cursor; the borrowed entry is dropped at the semicolon so
        // resolve_current can re-borrow self.
        self.history.back()?;
        self.resolve_current()
    }

    /// Moves history forward one entry and re-resolves it, or returns `None`
    /// when already at the end (spec §6.13).
    ///
    /// Like [`back`](Router::back), the entry is re-resolved against the current
    /// registry, so a now-unregistered pattern yields a
    /// [`NavigationResult::NotFound`].
    pub fn forward(&mut self) -> Option<NavigationResult> {
        // Move the cursor; the borrowed entry is dropped at the semicolon so
        // resolve_current can re-borrow self.
        self.history.forward()?;
        self.resolve_current()
    }

    /// Re-resolves the current history entry's route against the registry.
    ///
    /// Returns `None` only when there is no current entry (empty history).
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

    /// Returns the current history entry, or `None` when history is empty.
    #[must_use]
    pub fn current(&self) -> Option<&NavigationEntry> {
        self.history.current()
    }

    /// Returns whether a [`back`](Router::back) move is possible.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        self.history.can_go_back()
    }

    /// Returns whether a [`forward`](Router::forward) move is possible.
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        self.history.can_go_forward()
    }

    /// Returns the pattern-to-target registry this router resolves against.
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
