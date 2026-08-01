#![doc = include_str!("task_priority.doc/_module.md")]

#[doc = include_str!("task_priority.doc/TaskPriority.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    #[doc = include_str!("task_priority.doc/Background.md")]
    Background,
    #[doc = include_str!("task_priority.doc/Normal.md")]
    Normal,
    #[doc = include_str!("task_priority.doc/Interactive.md")]
    Interactive,
}
