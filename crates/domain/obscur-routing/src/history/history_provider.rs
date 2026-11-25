//! The replaceable history port and its in-memory reference impl (spec §6.13).

use crate::history::NavigationEntry;
use crate::route::{Route, RouteState};

#[doc = include_str!("../../docs/api/history/history_provider.doc/HistoryProvider.md")]
pub trait HistoryProvider: std::fmt::Debug {
    #[doc = include_str!("../../docs/api/history/history_provider.doc/push.md")]
    fn push(&mut self, route: Route, state: RouteState);

    #[doc = include_str!("../../docs/api/history/history_provider.doc/replace.md")]
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
