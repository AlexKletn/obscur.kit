#![doc = include_str!("../../docs/api/clipboard/clipboard_provider.doc/_module.md")]

use super::{ClipboardRequest, ClipboardResult};

#[doc = include_str!("../../docs/api/clipboard/clipboard_provider.doc/ClipboardProvider.md")]
pub trait ClipboardProvider {
    #[doc = include_str!("../../docs/api/clipboard/clipboard_provider.doc/execute.md")]
    fn execute(&mut self, request: ClipboardRequest) -> ClipboardResult;

    /// Returns whether a system clipboard is reachable through this provider
    /// (spec §6.20).
    #[must_use]
    fn is_available(&self) -> bool;
}
