#![doc = include_str!("history_provider.doc/_module.md")]

use crate::history::NavigationEntry;
use crate::route::{Route, RouteState};

#[doc = include_str!("history_provider.doc/HistoryProvider.md")]
pub trait HistoryProvider: std::fmt::Debug {
    #[doc = include_str!("history_provider.doc/push.md")]
    fn push(&mut self, route: Route, state: RouteState);

    #[doc = include_str!("history_provider.doc/replace.md")]
    fn replace(&mut self, route: Route, state: RouteState);

    #[doc = include_str!("history_provider.doc/back.md")]
    fn back(&mut self) -> Option<&NavigationEntry>;

    #[doc = include_str!("history_provider.doc/forward.md")]
    fn forward(&mut self) -> Option<&NavigationEntry>;

    #[doc = include_str!("history_provider.doc/current.md")]
    #[must_use]
    fn current(&self) -> Option<&NavigationEntry>;

    #[doc = include_str!("history_provider.doc/can_go_back.md")]
    #[must_use]
    fn can_go_back(&self) -> bool;

    #[doc = include_str!("history_provider.doc/can_go_forward.md")]
    #[must_use]
    fn can_go_forward(&self) -> bool;

    #[doc = include_str!("history_provider.doc/len.md")]
    #[must_use]
    fn len(&self) -> usize;

    #[doc = include_str!("history_provider.doc/is_empty.md")]
    #[must_use]
    fn is_empty(&self) -> bool;
}

#[doc = include_str!("history_provider.doc/MemoryHistory.md")]
#[derive(Debug, Default, Clone)]
pub struct MemoryHistory {
    #[doc = include_str!("history_provider.doc/entries.md")]
    entries: Vec<NavigationEntry>,
    #[doc = include_str!("history_provider.doc/cursor.md")]
    cursor: usize,
    #[doc = include_str!("history_provider.doc/next_id.md")]
    next_id: u64,
}

impl MemoryHistory {
    #[doc = include_str!("history_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("history_provider.doc/allocate_id.md")]
    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl HistoryProvider for MemoryHistory {
    fn push(&mut self, route: Route, state: RouteState) {
        let id = self.allocate_id();
        let entry = NavigationEntry::new(id, route).with_state(state);
        if self.entries.is_empty() {
            self.entries.push(entry);
            self.cursor = 0;
        } else {
            // Truncate the forward history, then append and point at the new tail.
            self.entries.truncate(self.cursor + 1);
            self.entries.push(entry);
            self.cursor = self.entries.len() - 1;
        }
    }

    fn replace(&mut self, route: Route, state: RouteState) {
        let id = self.allocate_id();
        let entry = NavigationEntry::new(id, route).with_state(state);
        if self.entries.is_empty() {
            self.entries.push(entry);
            self.cursor = 0;
        } else {
            self.entries[self.cursor] = entry;
        }
    }

    fn back(&mut self) -> Option<&NavigationEntry> {
        if self.can_go_back() {
            self.cursor -= 1;
            self.entries.get(self.cursor)
        } else {
            None
        }
    }

    fn forward(&mut self) -> Option<&NavigationEntry> {
        if self.can_go_forward() {
            self.cursor += 1;
            self.entries.get(self.cursor)
        } else {
            None
        }
    }

    fn current(&self) -> Option<&NavigationEntry> {
        self.entries.get(self.cursor)
    }

    fn can_go_back(&self) -> bool {
        !self.entries.is_empty() && self.cursor > 0
    }

    fn can_go_forward(&self) -> bool {
        !self.entries.is_empty() && self.cursor + 1 < self.entries.len()
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time proof the port is object-safe: adapters are stored as
    /// `Box<dyn HistoryProvider>`.
    #[test]
    fn history_provider_is_object_safe() {
        let history: Box<dyn HistoryProvider> = Box::new(MemoryHistory::new());
        assert!(history.is_empty());
    }

    #[test]
    fn push_advances_current_and_len() {
        let mut history = MemoryHistory::new();
        assert!(history.current().is_none());
        assert_eq!(history.len(), 0);
        assert!(history.is_empty());

        history.push(Route::parse("/a"), RouteState::empty());
        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());
        assert_eq!(history.current().unwrap().route().path(), "/a");
        assert!(!history.can_go_back());
        assert!(!history.can_go_forward());

        history.push(Route::parse("/b"), RouteState::empty());
        assert_eq!(history.len(), 2);
        assert_eq!(history.current().unwrap().route().path(), "/b");
        assert!(history.can_go_back());
        assert!(!history.can_go_forward());
    }

    #[test]
    fn back_and_forward_move_the_cursor() {
        let mut history = MemoryHistory::new();
        history.push(Route::parse("/a"), RouteState::empty());
        history.push(Route::parse("/b"), RouteState::empty());

        assert_eq!(history.back().unwrap().route().path(), "/a");
        assert!(history.can_go_forward());
        assert!(!history.can_go_back());
        // Already at the start: another back is a no-op returning None.
        assert!(history.back().is_none());

        assert_eq!(history.forward().unwrap().route().path(), "/b");
        // Already at the end: another forward is a no-op returning None.
        assert!(history.forward().is_none());
    }

    #[test]
    fn push_truncates_forward_history() {
        // push A, push B, back, push C -> B (the forward entry) is discarded.
        let mut history = MemoryHistory::new();
        history.push(Route::parse("/a"), RouteState::empty());
        history.push(Route::parse("/b"), RouteState::empty());
        history.back();
        history.push(Route::parse("/c"), RouteState::empty());

        assert_eq!(history.len(), 2);
        assert_eq!(history.current().unwrap().route().path(), "/c");
        // Forward is gone; the stack is [A, C].
        assert!(!history.can_go_forward());
        assert_eq!(history.back().unwrap().route().path(), "/a");
        assert_eq!(history.forward().unwrap().route().path(), "/c");
    }

    #[test]
    fn replace_overwrites_current_and_assigns_new_id() {
        let mut history = MemoryHistory::new();
        history.push(Route::parse("/a"), RouteState::empty());
        let first_id = history.current().unwrap().id();

        history.replace(Route::parse("/b"), RouteState::new("k", "v"));
        assert_eq!(history.len(), 1);
        let current = history.current().unwrap();
        assert_eq!(current.route().path(), "/b");
        assert_eq!(current.state().kind(), "k");
        assert_ne!(current.id(), first_id);
    }

    #[test]
    fn replace_on_empty_acts_like_push() {
        let mut history = MemoryHistory::new();
        history.replace(Route::parse("/a"), RouteState::empty());
        assert_eq!(history.len(), 1);
        assert_eq!(history.current().unwrap().route().path(), "/a");
    }

    #[test]
    fn ids_are_monotonic() {
        let mut history = MemoryHistory::new();
        history.push(Route::parse("/a"), RouteState::empty());
        let a = history.current().unwrap().id();
        history.push(Route::parse("/b"), RouteState::empty());
        let b = history.current().unwrap().id();
        assert_ne!(a, b);
        assert!(b > a);
    }
}
