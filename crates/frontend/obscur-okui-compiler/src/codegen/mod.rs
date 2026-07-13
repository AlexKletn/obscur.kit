//! Generating ordinary Rust that builds a runtime desired-view tree from
//! compiled UI IR at compile time (spec §2.8, §2.15).
//!
//! [`ViewCodegen`] is the ahead-of-time counterpart of the runtime
//! [`UiIrViewBridge`](crate::bridge::UiIrViewBridge): it lowers a
//! [`UiIrDocument`](obscur_ui_ir::document::UiIrDocument) once into Rust source
//! constructing the identical [`View`](obscur_view::view::View), so a `.okui`
//! document compiles to plain code with no frontend machinery left at runtime.
mod view_codegen;
pub use view_codegen::ViewCodegen;
