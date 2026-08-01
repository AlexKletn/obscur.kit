#![doc = include_str!("mod.doc/_module.md")]
mod backpressure_policy;
pub use backpressure_policy::{BackpressureDecision, BackpressurePolicy};
mod frame_scheduler;
pub use frame_scheduler::FrameScheduler;
mod scheduler_priority;
pub use scheduler_priority::SchedulerPriority;
mod scheduler_queue;
pub use scheduler_queue::SchedulerQueue;
mod starvation_guard;
pub use starvation_guard::StarvationGuard;
