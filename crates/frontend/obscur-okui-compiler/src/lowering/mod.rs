#![doc = include_str!("../../docs/api/lowering/mod.doc/_module.md")]

mod expression_lowering;
pub use expression_lowering::ExpressionLowering;
mod source_map_builder;
pub use source_map_builder::SourceMapBuilder;
mod ui_ir_lowering;
pub use ui_ir_lowering::UiIrLowering;
mod lowering_error;
pub use lowering_error::LoweringError;
// Crate-internal: the two readings of an authored attribute the lowering already
// had to define — the local event name after `on:`, and the raw text of a value.
// Re-exported so the validators check the *same* strings the IR is built from
// rather than each keeping a copy of the extraction. See their own docs.
pub(crate) use ui_ir_lowering::{attribute_value_text, event_local_name};
