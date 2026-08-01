#![doc = include_str!("mod.doc/_module.md")]
mod ui_ir_validator;
pub use ui_ir_validator::UiIrValidator;
mod validation_context;
pub use validation_context::ValidationContext;
mod validation_error;
pub use validation_error::UiIrValidationError;
mod validation_result;
pub use validation_result::UiIrValidationResult;
