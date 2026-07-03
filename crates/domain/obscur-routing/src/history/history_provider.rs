//! The replaceable history port and its in-memory reference impl (spec §6.13).

use crate::history::NavigationEntry;
use crate::route::{Route, RouteState};

/// The replaceable port for a navigation history stack (spec §6.13 history).
///
/// A history provider owns the ordered stack of visited entries and a cursor
/// marking the current one. The domain drives navigation through this trait and
/// stores it as `Box<dyn HistoryProvider>`, so it is deliberately **object-safe**
/// — no generic methods and no `Self`-returning methods. The
/// [`std::fmt::Debug`] supertrait lets a boxed provider be `Debug` (so the
/// composing `Router` can derive it).
///
/// Mutating navigation takes `&mut self`; queries take `&self`. The colocated
/// [`MemoryHistory`] is the reference in-memory implementation and documents the
/// contract of each method concretely.
pub trait HistoryProvider: std::fmt::Debug {
    /// Pushes a new entry for `route` with `state`, assigning a fresh id.
    ///
    /// Standard browser semantics: pushing truncates any forward history (every
    /// entry after the current cursor) before appending, so a push after going
    /// back discards the entries that were ahead. The new entry becomes current.
    fn push(&mut self, route: Route, state: RouteState);

    /// Replaces the current entry in place with a new entry for `route` and
    /// `state`, assigning a fresh id.
    ///
    /// The cursor and the rest of the stack are unchanged; only the current
    /// entry is overwritten. When the history is empty there is no current entry
    /// to replace, so this behaves like [`push`](HistoryProvider::push).
    fn replace(&mut self, route: Route, state: RouteState);

    /// Moves the cursor back one entry and returns the new current entry, or
    /// `None` when already at the start of history.
    fn back(&mut self) -> Option<&NavigationEntry>;

    /// Moves the cursor forward one entry and returns the new current entry, or
    /// `None` when already at the end of history.
    fn forward(&mut self) -> Option<&NavigationEntry>;

    /// Returns the current entry, or `None` when the history is empty.
    #[must_use]
    fn current(&self) -> Option<&NavigationEntry>;

    /// Returns whether a [`back`](HistoryProvider::back) move is possible.
    #[must_use]
    fn can_go_back(&self) -> bool;

    /// Returns whether a [`forward`](HistoryProvider::forward) move is possible.
    #[must_use]
    fn can_go_forward(&self) -> bool;

    /// Returns the number of entries in the history stack.
    #[must_use]
    fn len(&self) -> usize;

    /// Returns whether the history stack holds no entries.
    #[must_use]
    fn is_empty(&self) -> bool;
}

/// The reference, in-memory [`HistoryProvider`] (spec §6.13).
///
/// `MemoryHistory` keeps the visited entries in a [`Vec`] and a `cursor` index
/// pointing at the current entry. The cursor is only meaningful when the stack
/// is non-empty; when `entries` is empty it is a dormant `0` and every query
/// guards on emptiness. Fresh ids come from a plain monotonic `next_id` counter
/// — there is no wall-clock or randomness, so the provider is fully
/// deterministic.
#[derive(Debug, Default, Clone)]
pub struct MemoryHistory {
    /// The visited entries, oldest first.
    entries: Vec<NavigationEntry>,
    /// The index of the current entry; meaningful only when `entries` is
    /// non-empty.
    cursor: usize,
    /// The monotonic source of fresh entry ids.
    next_id: u64,
}

impl MemoryHistory {
    /// Creates an empty history with no entries.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Mints the next fresh entry id from the monotonic counter.
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
