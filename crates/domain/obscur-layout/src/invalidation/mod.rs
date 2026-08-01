#![doc = include_str!("mod.doc/_module.md")]
mod cache_entry;
pub use cache_entry::CacheEntry;
mod layout_cache;
pub use layout_cache::LayoutCache;
mod layout_invalidation;
pub use layout_invalidation::LayoutInvalidation;
mod layout_invalidation_reason;
pub use layout_invalidation_reason::LayoutInvalidationReason;
mod propagation_result;
pub use propagation_result::PropagationResult;
