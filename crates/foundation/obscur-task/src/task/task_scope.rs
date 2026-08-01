#![doc = include_str!("task_scope.doc/_module.md")]

#[doc = include_str!("task_scope.doc/TaskScope.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskScope {
    #[doc = include_str!("task_scope.doc/Component.md")]
    Component,
    #[doc = include_str!("task_scope.doc/Window.md")]
    Window,
    #[doc = include_str!("task_scope.doc/Runtime.md")]
    Runtime,
}
