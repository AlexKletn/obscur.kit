#![doc = include_str!("navigation_entry.doc/_module.md")]

use crate::route::{Route, RouteState};

#[doc = include_str!("navigation_entry.doc/NavigationEntry.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationEntry {
    #[doc = include_str!("navigation_entry.doc/route.md")]
    route: Route,
    #[doc = include_str!("navigation_entry.doc/state.md")]
    state: RouteState,
    #[doc = include_str!("navigation_entry.doc/id.md")]
    id: u64,
}

impl NavigationEntry {
    #[doc = include_str!("navigation_entry.doc/new.md")]
    #[must_use]
    pub fn new(id: u64, route: Route) -> Self {
        Self {
            route,
            state: RouteState::empty(),
            id,
        }
    }

    #[doc = include_str!("navigation_entry.doc/with_state.md")]
    #[must_use]
    pub fn with_state(mut self, state: RouteState) -> Self {
        self.state = state;
        self
    }

    #[doc = include_str!("navigation_entry.doc/route_2.md")]
    #[must_use]
    pub fn route(&self) -> &Route {
        &self.route
    }

    #[doc = include_str!("navigation_entry.doc/state_2.md")]
    #[must_use]
    pub fn state(&self) -> &RouteState {
        &self.state
    }

    #[doc = include_str!("navigation_entry.doc/id_2.md")]
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
