#![doc = include_str!("dialog_provider.doc/_module.md")]

use std::collections::VecDeque;

use obscur_capability::SupportLevel;

use crate::dialog::{DialogRequest, DialogResult};

#[doc = include_str!("dialog_provider.doc/DialogProvider.md")]
pub trait DialogProvider {
    #[doc = include_str!("dialog_provider.doc/origin.md")]
    fn origin(&self) -> &str;

    #[doc = include_str!("dialog_provider.doc/present.md")]
    fn present(&mut self, request: DialogRequest) -> DialogResult;

    #[doc = include_str!("dialog_provider.doc/supports.md")]
    #[must_use]
    fn supports(&self, request: &DialogRequest) -> SupportLevel;
}

#[doc = include_str!("dialog_provider.doc/NullDialogProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct NullDialogProvider;

impl DialogProvider for NullDialogProvider {
    fn origin(&self) -> &'static str {
        "null"
    }

    fn present(&mut self, _request: DialogRequest) -> DialogResult {
        // Honest: a null provider can present no dialog.
        DialogResult::Unsupported
    }

    fn supports(&self, _request: &DialogRequest) -> SupportLevel {
        SupportLevel::Unavailable
    }
}

#[doc = include_str!("dialog_provider.doc/ScriptedDialogProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct ScriptedDialogProvider {
    queue: VecDeque<DialogResult>,
}

impl ScriptedDialogProvider {
    #[doc = include_str!("dialog_provider.doc/new.md")]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[doc = include_str!("dialog_provider.doc/push_result.md")]
    pub fn push_result(&mut self, result: DialogResult) {
        self.queue.push_back(result);
    }
}

impl DialogProvider for ScriptedDialogProvider {
    fn origin(&self) -> &'static str {
        "scripted"
    }

    fn present(&mut self, _request: DialogRequest) -> DialogResult {
        self.queue.pop_front().unwrap_or(DialogResult::Dismissed)
    }

    fn supports(&self, _request: &DialogRequest) -> SupportLevel {
        SupportLevel::Emulated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_provider_is_object_safe() {
        let _: Box<dyn DialogProvider> = Box::new(NullDialogProvider);
    }

    #[test]
    fn null_provider_is_honestly_unsupported() {
        let mut provider = NullDialogProvider;
        assert_eq!(provider.origin(), "null");
        assert_eq!(
            provider.present(DialogRequest::open_file()),
            DialogResult::Unsupported
        );
        assert_eq!(
            provider.supports(&DialogRequest::open_file()),
            SupportLevel::Unavailable
        );
    }

    #[test]
    fn scripted_provider_pops_in_fifo_order_then_dismisses() {
        let mut provider = ScriptedDialogProvider::new();
        assert_eq!(provider.origin(), "scripted");
        assert_eq!(
            provider.supports(&DialogRequest::pick_folder()),
            SupportLevel::Emulated
        );

        provider.push_result(DialogResult::Confirmed(true));
        provider.push_result(DialogResult::Denied);

        assert_eq!(
            provider.present(DialogRequest::confirm("t", "b")),
            DialogResult::Confirmed(true)
        );
        assert_eq!(
            provider.present(DialogRequest::confirm("t", "b")),
            DialogResult::Denied
        );
        // Drained: falls back to Dismissed.
        assert_eq!(
            provider.present(DialogRequest::confirm("t", "b")),
            DialogResult::Dismissed
        );
    }
}
