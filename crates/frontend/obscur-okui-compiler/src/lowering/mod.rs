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
