//! Web/WASM system-service providers backed by browser APIs (spec §6.9 dialogs,
//! §6.11 shell, §6.12 notifications; §2.7 target-specific providers).
//!
//! Each provider implements one `obscur_system` port over a `web-sys` browser
//! binding: [`WebDialogProvider`] over `window.confirm`/`window.alert`,
//! [`WebShellProvider`] over `window.open`, and [`WebNotificationProvider`] over
//! the `Notification` API. Every browser call is gated behind
//! `#[cfg(target_arch = "wasm32")]`; on the default (non-wasm) host build each
//! provider falls back to the port's honest "unsupported" outcome, so the code
//! compiles and its capability contract stays truthful on both targets.
//! Asynchronous browser facilities (file-input dialogs, the async notification
//! permission prompt, the Badging API) are documented future refinements.

mod web_dialog_provider;
pub use web_dialog_provider::WebDialogProvider;
mod web_notification_provider;
pub use web_notification_provider::WebNotificationProvider;
mod web_shell_provider;
pub use web_shell_provider::WebShellProvider;
