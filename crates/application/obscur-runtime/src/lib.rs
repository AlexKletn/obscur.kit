#![doc = include_str!("lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

mod runtime;

pub use runtime::{
    AppCommands, CanvasPainter, CommandHandler, CommandTarget, FrameBudget, ImageCache,
    InputDispatch, InputEvent, IntoCommandHandler, KeyDispatch, PainterKey, PointerDispatch,
    SemanticTreeBuilder, UiRuntime, WithEvent, WithoutEvent,
};

pub mod component;
pub mod frame;
pub mod invalidation;
pub mod lifecycle;
pub mod orchestration;
pub mod quality;
pub mod recovery;
pub mod scheduler;
pub mod transaction;
