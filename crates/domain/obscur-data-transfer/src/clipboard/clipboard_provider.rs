//! The replaceable clipboard port and its in-process reference implementation
//! (spec §6.20 data transfer — clipboard).
//!
//! [`ClipboardProvider`] is the hexagonal port through which the pure
//! data-transfer domain drives a real system clipboard: an interface adapter
//! implements it once per platform and the domain holds it as
//! `Box<dyn ClipboardProvider>`. The colocated [`MemoryClipboard`] is a
//! deterministic, in-process reference implementation — the honest default for
//! headless runs and tests. System clipboard *history* is explicitly out of
//! scope (spec §6.20): a provider models only the current contents.

use super::{ClipboardRequest, ClipboardResult};
use crate::offer::DataOffer;

/// The replaceable port for a system clipboard (spec §6.20 data transfer —
/// clipboard).
///
/// Every clipboard operation the domain performs funnels through this trait's
/// single [`execute`](ClipboardProvider::execute) entry point, which dispatches
/// copy/cut/paste/clear and reports an honest [`ClipboardResult`]. The trait is
/// deliberately **object-safe** — no generic or `Self`-returning methods — so
/// adapters can be held polymorphically behind `Box<dyn ClipboardProvider>`.
pub trait ClipboardProvider {
    /// Performs `request` against the clipboard, reporting the honest outcome
    /// (spec §6.20).
    ///
    /// This is the unified copy/cut/paste/clear entry point; it never panics on
    /// an unsatisfiable request, returning the appropriate non-success
    /// [`ClipboardResult`] instead.
    fn execute(&mut self, request: ClipboardRequest) -> ClipboardResult;

    /// Returns whether a system clipboard is reachable through this provider
    /// (spec §6.20).
    #[must_use]
    fn is_available(&self) -> bool;
}

/// A deterministic, in-process reference [`ClipboardProvider`] (spec §6.20).
///
/// `MemoryClipboard` holds only the current clipboard contents — the last
/// written [`DataOffer`] — in memory, with no platform dependency. It is the
/// honest default for headless environments and tests, and documents the port's
/// contract concretely. System clipboard *history* is out of scope (spec §6.20);
/// this holds a single slot. At the storage level a cut is treated exactly like
/// a copy — the source-side deletion a [`Cut`](super::ClipboardWriteMode::Cut)
/// implies is the caller's concern, not the clipboard's.
#[derive(Debug, Default, Clone)]
pub struct MemoryClipboard {
    /// The current clipboard contents: the last written offer, if any.
    contents: Option<DataOffer>,
}

impl MemoryClipboard {
    /// Creates an empty in-process clipboard (spec §6.20).
    #[must_use]
    pub const fn new() -> Self {
        Self { contents: None }
    }
}

impl ClipboardProvider for MemoryClipboard {
    fn execute(&mut self, request: ClipboardRequest) -> ClipboardResult {
        match request {
            ClipboardRequest::Write { payload, .. } => {
                if payload.is_empty() {
                    ClipboardResult::Empty
                } else {
                    self.contents = Some(payload.offer().clone());
                    ClipboardResult::Written
                }
            }
            ClipboardRequest::Read { mimes } => match &self.contents {
                None => ClipboardResult::Empty,
                Some(offer) => {
                    let has_requested_format =
                        mimes.is_empty() || mimes.iter().any(|mime| offer.has_mime(mime));
                    if has_requested_format {
                        ClipboardResult::Read(offer.clone())
                    } else {
                        ClipboardResult::Empty
                    }
                }
            },
            ClipboardRequest::Clear => {
                self.contents = None;
                ClipboardResult::Cleared
            }
        }
    }

    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offer::MimeType;
    use crate::transfer::TransferPayload;

    /// Compile-time proof the port is object-safe: adapters must be storable as
    /// `Box<dyn ClipboardProvider>`.
    #[test]
    fn clipboard_provider_is_object_safe() {
        let provider: Box<dyn ClipboardProvider> = Box::new(MemoryClipboard::new());
        assert!(provider.is_available());
    }

    #[test]
    fn write_then_read_round_trips_a_text_offer() {
        let mut clipboard = MemoryClipboard::new();
        let payload = TransferPayload::new(DataOffer::text("hi"));

        assert_eq!(
            clipboard.execute(ClipboardRequest::copy(payload)),
            ClipboardResult::Written
        );
        assert_eq!(
            clipboard.execute(ClipboardRequest::read_any()),
            ClipboardResult::Read(DataOffer::text("hi"))
        );
    }

    #[test]
    fn cut_stores_like_copy() {
        let mut clipboard = MemoryClipboard::new();
        let payload = TransferPayload::new(DataOffer::text("hi"));
        assert_eq!(
            clipboard.execute(ClipboardRequest::cut(payload)),
            ClipboardResult::Written
        );
        assert_eq!(
            clipboard.execute(ClipboardRequest::read_any()),
            ClipboardResult::Read(DataOffer::text("hi"))
        );
    }

    #[test]
    fn writing_an_empty_payload_reports_empty() {
        let mut clipboard = MemoryClipboard::new();
        let empty = TransferPayload::new(DataOffer::new());
        assert_eq!(
            clipboard.execute(ClipboardRequest::copy(empty)),
            ClipboardResult::Empty
        );
    }

    #[test]
    fn read_on_empty_clipboard_is_empty() {
        let mut clipboard = MemoryClipboard::new();
        assert_eq!(
            clipboard.execute(ClipboardRequest::read_any()),
            ClipboardResult::Empty
        );
    }

    #[test]
    fn read_with_non_matching_mime_is_empty() {
        let mut clipboard = MemoryClipboard::new();
        clipboard.execute(ClipboardRequest::copy(TransferPayload::new(DataOffer::text("hi"))));
        assert_eq!(
            clipboard.execute(ClipboardRequest::read(vec![MimeType::image_png()])),
            ClipboardResult::Empty
        );
        // A matching request still reads.
        assert_eq!(
            clipboard.execute(ClipboardRequest::read(vec![MimeType::text_plain()])),
            ClipboardResult::Read(DataOffer::text("hi"))
        );
    }

    #[test]
    fn clear_empties_so_subsequent_read_is_empty() {
        let mut clipboard = MemoryClipboard::new();
        clipboard.execute(ClipboardRequest::copy(TransferPayload::new(DataOffer::text("hi"))));
        assert_eq!(
            clipboard.execute(ClipboardRequest::clear()),
            ClipboardResult::Cleared
        );
        assert_eq!(
            clipboard.execute(ClipboardRequest::read_any()),
            ClipboardResult::Empty
        );
    }

    #[test]
    fn is_available_is_true() {
        assert!(MemoryClipboard::new().is_available());
        assert!(MemoryClipboard::default().is_available());
    }
}
