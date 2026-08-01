#![doc = include_str!("web_dialog_provider.doc/_module.md")]

use obscur_capability::SupportLevel;
use obscur_system::dialog::{DialogProvider, DialogRequest, DialogResult};

#[doc = include_str!("web_dialog_provider.doc/WebDialogProvider.md")]
#[derive(Debug, Clone, Default)]
pub struct WebDialogProvider;

impl WebDialogProvider {
    #[doc = include_str!("web_dialog_provider.doc/new.md")]
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
