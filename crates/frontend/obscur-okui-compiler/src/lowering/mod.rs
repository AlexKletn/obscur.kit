//! Lowering the `.okui` AST into UI IR (spec §2.9, §2.15).
//!
//! [`UiIrLowering`] drives the pass: it walks a parsed
//! [`Document`](obscur_okui_syntax::ast::Document) and produces a
//! [`UiIrDocument`](obscur_ui_ir::document::UiIrDocument), delegating value
//! classification to [`ExpressionLowering`] and recording node-id → source
//! spans through [`SourceMapBuilder`]. [`LoweringError`] reports why a document
//! could not be lowered.

mod expression_lowering;
pub use expression_lowering::ExpressionLowering;
mod source_map_builder;
pub use source_map_builder::SourceMapBuilder;
mod ui_ir_lowering;
pub use ui_ir_lowering::{LoweringError, UiIrLowering};
// Crate-internal: the two readings of an authored attribute the lowering already
// had to define — the local event name after `on:`, and the raw text of a value.
// Re-exported so the validators check the *same* strings the IR is built from
// rather than each keeping a copy of the extraction. See their own docs.
pub(crate) use ui_ir_lowering::{attribute_value_text, event_local_name};
