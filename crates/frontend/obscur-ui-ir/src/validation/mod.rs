//! Structural validation of UI IR documents: the [`UiIrValidator`] walks a
//! document and accumulates [`UiIrValidationError`]s into a
//! [`UiIrValidationResult`], threading a [`ValidationContext`] for cross-node
//! state (spec §2.5 validation).
mod ui_ir_validator;
pub use ui_ir_validator::UiIrValidator;
mod validation_context;
pub use validation_context::ValidationContext;
mod validation_error;
pub use validation_error::UiIrValidationError;
mod validation_result;
pub use validation_result::UiIrValidationResult;
