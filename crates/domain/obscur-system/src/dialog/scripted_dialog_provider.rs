use super::{DialogProvider, DialogRequest, DialogResult};
use obscur_capability::SupportLevel;
use std::collections::VecDeque;

#[doc = include_str!("../../docs/api/dialog/dialog_provider.doc/ScriptedDialogProvider.md")]
#[derive(Debug, Default, Clone)]
pub struct ScriptedDialogProvider {
    queue: VecDeque<DialogResult>,
}

impl ScriptedDialogProvider {
    /// Creates an empty scripted provider (spec §6.9).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueues `result` to be returned by a later [`Self::present`] call
    /// (spec §6.9).
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
