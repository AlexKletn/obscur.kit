#![doc = include_str!("mod.doc/_module.md")]
mod history_provider;
pub use history_provider::{HistoryProvider, MemoryHistory};
mod navigation_direction;
pub use navigation_direction::NavigationDirection;
mod navigation_entry;
pub use navigation_entry::NavigationEntry;
