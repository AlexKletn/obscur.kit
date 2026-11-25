//! History module.
mod history_provider;
pub use history_provider::HistoryProvider;
mod memory_history;
pub use memory_history::MemoryHistory;
mod navigation_direction;
pub use navigation_direction::NavigationDirection;
mod navigation_entry;
pub use navigation_entry::NavigationEntry;
