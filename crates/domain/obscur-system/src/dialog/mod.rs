//! Dialog module.
mod dialog_provider;
pub use dialog_provider::DialogProvider;
mod null_dialog_provider;
pub use null_dialog_provider::NullDialogProvider;
mod scripted_dialog_provider;
pub use scripted_dialog_provider::ScriptedDialogProvider;
mod dialog_request;
pub use dialog_request::DialogRequest;
mod message_level;
pub use message_level::MessageLevel;
mod file_filter;
pub use file_filter::FileFilter;
mod dialog_result;
pub use dialog_result::DialogResult;
