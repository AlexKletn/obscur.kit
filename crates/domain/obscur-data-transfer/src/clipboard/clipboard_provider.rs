#![doc = include_str!("clipboard_provider.doc/_module.md")]

use super::{ClipboardRequest, ClipboardResult};
use crate::offer::DataOffer;

#[doc = include_str!("clipboard_provider.doc/ClipboardProvider.md")]
pub trait ClipboardProvider {
    #[doc = include_str!("clipboard_provider.doc/execute.md")]
    fn execute(&mut self, request: ClipboardRequest) -> ClipboardResult;

    #[doc = include_str!("clipboard_provider.doc/is_available.md")]
    #[must_use]
    fn is_available(&self) -> bool;
}

#[doc = include_str!("clipboard_provider.doc/MemoryClipboard.md")]
#[derive(Debug, Default, Clone)]
pub struct MemoryClipboard {
    #[doc = include_str!("clipboard_provider.doc/contents.md")]
    contents: Option<DataOffer>,
}

impl MemoryClipboard {
    #[doc = include_str!("clipboard_provider.doc/new.md")]
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
        clipboard.execute(ClipboardRequest::copy(TransferPayload::new(
            DataOffer::text("hi"),
        )));
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
        clipboard.execute(ClipboardRequest::copy(TransferPayload::new(
            DataOffer::text("hi"),
        )));
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
