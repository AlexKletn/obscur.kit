//! Runtime module.
mod ui_runtime;

pub use ui_runtime::UiRuntime;

mod text_edit_session;

pub(crate) use text_edit_session::TextEditSession;

mod pointer_dispatch;

pub use pointer_dispatch::PointerDispatch;

mod key_dispatch;

pub use key_dispatch::KeyDispatch;

mod semantic_builder;

pub use semantic_builder::SemanticTreeBuilder;

mod frame_plan;

pub(crate) use frame_plan::FramePlan;

mod frame_budget;

pub use frame_budget::FrameBudget;

mod runtime_config;
mod runtime_event;
mod runtime_handle;
mod runtime_state;
