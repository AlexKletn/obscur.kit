//! A single entry in the navigation history stack (spec §6.13).

use crate::route::{Route, RouteState};

#[doc = include_str!("../../docs/api/history/navigation_entry.doc/NavigationEntry.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationEntry {
    /// The route this entry navigated to.
    route: Route,
    /// The opaque, application-defined state captured for this entry.
    state: RouteState,
    /// The monotonic identity assigned by the history provider.
    id: u64,
}

impl NavigationEntry {
    /// Creates an entry for `route` with the given `id` and empty state.
    #[must_use]
    pub fn new(id: u64, route: Route) -> Self {
        Self {
            route,
            state: RouteState::empty(),
            id,
        }
    }

    /// Returns this entry with its state replaced by `state` (builder style).
    #[must_use]
    pub fn with_state(mut self, state: RouteState) -> Self {
        self.state = state;
        self
    }

    /// Returns the route this entry navigated to.
    #[must_use]
    pub fn route(&self) -> &Route {
        &self.route
    }

    /// Returns the opaque state captured for this entry.
    #[must_use]
    pub fn state(&self) -> &RouteState {
        &self.state
    }

    /// Returns the monotonic identity assigned by the history provider.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_empty_state() {
        let entry = NavigationEntry::new(7, Route::parse("/users/42"));
        assert_eq!(entry.id(), 7);
        assert_eq!(entry.route().path(), "/users/42");
        assert!(entry.state().is_empty());
    }

    #[test]
    fn with_state_attaches_state() {
        let entry = NavigationEntry::new(1, Route::new("/p"))
            .with_state(RouteState::new("scroll", "{\"y\":10}"));
        assert_eq!(entry.state().kind(), "scroll");
        assert_eq!(entry.state().payload(), "{\"y\":10}");
        assert!(!entry.state().is_empty());
        // The route and id are untouched by with_state.
        assert_eq!(entry.id(), 1);
        assert_eq!(entry.route().path(), "/p");
    }
}
