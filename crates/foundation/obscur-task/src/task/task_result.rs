#![doc = include_str!("task_result.doc/_module.md")]

use crate::task::{TaskId, TaskScope};

#[doc = include_str!("task_result.doc/TaskResult.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskResult<T> {
    #[doc = include_str!("task_result.doc/id.md")]
    id: TaskId,
    #[doc = include_str!("task_result.doc/scope.md")]
    scope: TaskScope,
    #[doc = include_str!("task_result.doc/payload.md")]
    payload: T,
}

impl<T> TaskResult<T> {
    #[doc = include_str!("task_result.doc/new.md")]
    #[must_use]
    pub const fn new(id: TaskId, scope: TaskScope, payload: T) -> Self {
        Self { id, scope, payload }
    }

    #[doc = include_str!("task_result.doc/id_2.md")]
    #[must_use]
    pub const fn id(&self) -> TaskId {
        self.id
    }

    #[doc = include_str!("task_result.doc/scope_2.md")]
    #[must_use]
    pub const fn scope(&self) -> TaskScope {
        self.scope
    }

    #[doc = include_str!("task_result.doc/payload_2.md")]
    #[must_use]
    pub const fn payload(&self) -> &T {
        &self.payload
    }

    #[doc = include_str!("task_result.doc/into_payload.md")]
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
