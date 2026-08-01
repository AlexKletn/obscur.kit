#![doc = include_str!("mod.doc/_module.md")]
mod task_id;
mod task_priority;
mod task_scope;

pub use task_id::TaskId;
pub use task_priority::TaskPriority;
pub use task_scope::TaskScope;

mod task_result;
pub use task_result::TaskResult;

// `StaleResult` was a skeleton placeholder for "stale work is discarded using
// generation/version IDs" (spec §3.3). That rule is real, but it is enforced
// where the identity that decides it lives — `ResourceHub::complete` rejects a
// job whose `ResourceVersion` is behind. A second, generic staleness type here
// would be a competing answer to the same question, so the placeholder is gone
// rather than given an invented contract.
