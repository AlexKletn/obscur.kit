//! A web [`DialogProvider`] backed by `window.confirm`/`window.alert`
//! (spec §6.9).
//!
//! [`WebDialogProvider`] implements the pure
//! [`DialogProvider`](obscur_system::dialog::DialogProvider) port over the
//! synchronous browser dialogs [`Window::confirm_with_message`](web_sys::Window::confirm_with_message)
//! and [`Window::alert_with_message`](web_sys::Window::alert_with_message). Only
//! message and confirmation dialogs have a synchronous browser primitive: file
//! and folder pickers require asynchronous `<input type="file">` DOM interaction,
//! which cannot be expressed through this synchronous port, so those requests are
//! reported [`SupportLevel::Unavailable`] / [`DialogResult::Unsupported`]. Wiring
//! them over an async surface is a documented future refinement.
//!
//! All browser calls are gated behind `#[cfg(target_arch = "wasm32")]`; on any
//! other target every request is [`DialogResult::Unsupported`], keeping the host
//! build honest.

use obscur_capability::SupportLevel;
use obscur_system::dialog::{DialogProvider, DialogRequest, DialogResult};

/// A web [`DialogProvider`] backed by `window.confirm`/`window.alert`
/// (spec §6.9).
///
/// The provider is stateless. [`Self::origin`] is `"web"`; [`Self::supports`]
/// reports [`SupportLevel::Native`] for message and confirmation dialogs and
/// [`SupportLevel::Unavailable`] for file/folder dialogs (which need async DOM
/// interaction); and [`Self::present`] drives the corresponding synchronous
/// browser dialog.
#[derive(Debug, Clone, Default)]
pub struct WebDialogProvider;

impl WebDialogProvider {
    /// Creates a web dialog provider (spec §6.9).
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl DialogProvider for WebDialogProvider {
    fn origin(&self) -> &'static str {
        "web"
    }

    fn present(&mut self, request: DialogRequest) -> DialogResult {
        #[cfg(target_arch = "wasm32")]
        {
            match request {
                DialogRequest::Confirm { title, body } => {
                    // A missing window or a browser refusal both read as "not
                    // confirmed" — the safe default for a confirmation.
                    let confirmed = web_sys::window().is_some_and(|w| {
                        w.confirm_with_message(&format!("{title}\n{body}"))
                            .unwrap_or(false)
                    });
                    DialogResult::Confirmed(confirmed)
                }
                DialogRequest::Message { title, body, .. } => {
                    // `alert` is purely informational; acknowledging it confirms.
                    if let Some(w) = web_sys::window() {
                        let _ = w.alert_with_message(&format!("{title}\n{body}"));
                    }
                    DialogResult::Confirmed(true)
                }
                // File / save / folder pickers need async DOM interaction.
                DialogRequest::OpenFile { .. }
                | DialogRequest::SaveFile { .. }
                | DialogRequest::PickFolder => DialogResult::Unsupported,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = request;
            DialogResult::Unsupported
        }
    }

    fn supports(&self, request: &DialogRequest) -> SupportLevel {
        #[cfg(target_arch = "wasm32")]
        {
            match request {
                // Backed by synchronous `window.confirm` / `window.alert`.
                DialogRequest::Confirm { .. } | DialogRequest::Message { .. } => {
                    SupportLevel::Native
                }
                // File dialogs need async `<input type=file>` DOM interaction,
                // which this synchronous port cannot express.
                DialogRequest::OpenFile { .. }
                | DialogRequest::SaveFile { .. }
                | DialogRequest::PickFolder => SupportLevel::Unavailable,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = request;
            SupportLevel::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_is_web() {
        assert_eq!(WebDialogProvider::new().origin(), "web");
    }

    #[test]
    fn present_confirm_is_unsupported_on_host_target() {
        let mut provider = WebDialogProvider::new();
        assert_eq!(
            provider.present(DialogRequest::confirm("Delete?", "Sure?")),
            DialogResult::Unsupported
        );
    }

    #[test]
    fn supports_is_unavailable_on_host_target() {
        let provider = WebDialogProvider::new();
        assert_eq!(
            provider.supports(&DialogRequest::confirm("t", "b")),
            SupportLevel::Unavailable
        );
        assert_eq!(
            provider.supports(&DialogRequest::open_file()),
            SupportLevel::Unavailable
        );
    }
}
