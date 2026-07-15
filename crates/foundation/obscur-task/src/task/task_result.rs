//! `TaskResult`: what a finished task hands back to the runtime.

use crate::task::{TaskId, TaskScope};

/// The typed message a finished task delivers (spec §3.3: "results return as
/// typed messages").
///
/// # Typed, not erased
///
/// `T` is the caller's own payload type — a decoded resource, a parsed document,
/// whatever the task produced. The envelope adds only what the *runtime* needs to
/// route it: which task it came from, and what that task was scoped to. Nothing
/// here is `Any`-erased or stringly-typed, so a receiver matches on a real Rust
/// value rather than re-parsing a message.
///
/// # It carries no generation
///
/// Deliberately. §3.3 also requires that "stale work is discarded using
/// generation/version IDs", but the ID that decides staleness belongs to the
/// *domain the work targets* — a resource load is stale against its
/// `ResourceVersion`, which the resource hub already checks on completion.
/// Putting a second, generic generation here would create a competing answer to
/// "is this stale", and the two could disagree. The payload carries its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskResult<T> {
    /// The task that produced this result.
    id: TaskId,
    /// What the task was scoped to — a caller drops a result whose scope has since
    /// been torn down (spec §3.3 "scoped tasks are cancelled automatically").
    scope: TaskScope,
    /// The task's own typed output.
    payload: T,
}

impl<T> TaskResult<T> {
    /// Wraps `payload` as the result of task `id`, scoped to `scope`.
    #[must_use]
    pub const fn new(id: TaskId, scope: TaskScope, payload: T) -> Self {
        Self { id, scope, payload }
    }

    /// The task that produced this result.
    #[must_use]
    pub const fn id(&self) -> TaskId {
        self.id
    }

    /// What the task was scoped to.
    #[must_use]
    pub const fn scope(&self) -> TaskScope {
        self.scope
    }

    /// The typed output.
    #[must_use]
    pub const fn payload(&self) -> &T {
        &self.payload
    }

    /// Takes the typed output, consuming the envelope.
    #[must_use]
    pub fn into_payload(self) -> T {
        self.payload
    }
}

#[cfg(test)]
mod tests {
    use super::TaskResult;
    use crate::task::{TaskId, TaskScope};

    #[test]
    fn it_carries_the_task_its_scope_and_a_typed_payload() {
        let result = TaskResult::new(TaskId::from_raw(7), TaskScope::Runtime, 42_u32);
        assert_eq!(result.id(), TaskId::from_raw(7));
        assert_eq!(result.scope(), TaskScope::Runtime);
        assert_eq!(*result.payload(), 42);
        assert_eq!(result.into_payload(), 42);
    }

    #[test]
    fn the_payload_stays_the_callers_own_type() {
        // Not erased: a receiver matches on a real value.
        #[derive(Debug, PartialEq)]
        enum Outcome {
            Loaded(String),
            Failed,
        }
        let result = TaskResult::new(
            TaskId::from_raw(1),
            TaskScope::Component,
            Outcome::Loaded("bytes".to_owned()),
        );
        assert_eq!(result.into_payload(), Outcome::Loaded("bytes".to_owned()));
    }
}
