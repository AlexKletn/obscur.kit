#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod cancellation;
mod port;
mod task;
mod time;

pub use cancellation::{CancellationReason, CancellationSource, CancellationToken};
pub use port::{BlockingExecutor, BoxFuture, LocalBoxFuture, LocalSpawner, Spawner, Timer};
pub use task::{TaskId, TaskPriority, TaskResult, TaskScope};
pub use time::{Clock, Deadline, Instant};
