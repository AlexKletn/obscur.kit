//! IR format versioning: the [`UiIrVersion`] format version and the
//! [`UiIrVersionRange`] of supported versions (spec §2.5).
mod ui_ir_version;
pub use ui_ir_version::UiIrVersion;
mod ui_ir_version_range;
pub use ui_ir_version_range::UiIrVersionRange;
