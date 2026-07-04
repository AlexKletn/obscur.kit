//! The IR document model: the [`UiIrDocument`] tree of [`UiIrNode`]s carrying
//! [`UiIrProperty`]/[`UiIrEvent`] bindings and [`UiIrExpression`] values
//! (spec §2.5).
mod ui_ir_document;
pub use ui_ir_document::UiIrDocument;
mod ui_ir_event;
pub use ui_ir_event::UiIrEvent;
mod ui_ir_expression;
pub use ui_ir_expression::{UiIrExpression, UiIrValue};
mod ui_ir_node;
pub use ui_ir_node::{UiIrNode, UiIrNodeKind};
mod ui_ir_property;
pub use ui_ir_property::UiIrProperty;
