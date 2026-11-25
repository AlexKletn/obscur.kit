//! The IR document model: the [`UiIrDocument`] tree of [`UiIrNode`]s carrying
//! [`UiIrProperty`]/[`UiIrEvent`] bindings and [`UiIrExpression`] values
//! (spec §2.5).
mod ui_ir_document;
pub use ui_ir_document::UiIrDocument;
mod ui_ir_event;
pub use ui_ir_event::UiIrEvent;
mod ui_ir_expression;
pub use ui_ir_expression::UiIrExpression;
mod ui_ir_value;
pub use ui_ir_value::UiIrValue;
mod ui_ir_node;
pub use ui_ir_node::UiIrNode;
mod ui_ir_node_kind;
pub use ui_ir_node_kind::UiIrNodeKind;
mod ui_ir_property;
pub use ui_ir_property::UiIrProperty;
