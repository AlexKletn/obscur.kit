#![doc = include_str!("mod.doc/_module.md")]

mod web_dialog_provider;
pub use web_dialog_provider::WebDialogProvider;
mod web_notification_provider;
pub use web_notification_provider::WebNotificationProvider;
mod web_shell_provider;
pub use web_shell_provider::WebShellProvider;
