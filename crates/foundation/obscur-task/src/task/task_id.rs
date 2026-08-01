#![doc = include_str!("task_id.doc/_module.md")]

#[doc = include_str!("task_id.doc/TaskId.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    #[doc = include_str!("task_id.doc/from_raw.md")]
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    #[doc = include_str!("task_id.doc/raw.md")]
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}
