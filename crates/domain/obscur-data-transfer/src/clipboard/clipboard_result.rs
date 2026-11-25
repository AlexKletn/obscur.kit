//! The typed outcome of a clipboard request (spec §6.20 data transfer —
//! clipboard).

use crate::offer::DataOffer;
use crate::transfer::TransferError;

#[doc = include_str!("../../docs/api/clipboard/clipboard_result.doc/ClipboardResult.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardResult {
    /// A paste succeeded, yielding this offer (spec §6.20 PASTE).
    Read(DataOffer),
    /// A copy or cut stored its payload (spec §6.20 COPY/CUT).
    Written,
    /// The clipboard was cleared (spec §6.20).
    Cleared,
    /// There was nothing on the clipboard to read (spec §6.20).
    Empty,
    /// A permission check refused clipboard access (spec §6.20 permissions).
    Denied,
    /// The requested format was unavailable, or the operation is unsupported
    /// (spec §6.20).
    Unsupported,
}

impl ClipboardResult {
    /// Returns whether this is a success outcome — a
    /// [`Read`](ClipboardResult::Read), [`Written`](ClipboardResult::Written),
    /// or [`Cleared`](ClipboardResult::Cleared) (spec §6.20).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Read(_) | Self::Written | Self::Cleared)
    }

    /// Returns the pasted [`DataOffer`] for a [`Read`](ClipboardResult::Read),
    /// or `None` otherwise (spec §6.20 PASTE).
    #[must_use]
    pub const fn offer(&self) -> Option<&DataOffer> {
        match self {
            Self::Read(offer) => Some(offer),
            _ => None,
        }
    }

    #[doc = include_str!("../../docs/api/clipboard/clipboard_result.doc/into_error.md")]
    #[must_use]
    pub fn into_error(self) -> Option<TransferError> {
        match self {
            Self::Read(_) | Self::Written | Self::Cleared => None,
            Self::Empty => Some(TransferError::Empty),
            Self::Denied => Some(TransferError::PermissionDenied),
            Self::Unsupported => Some(TransferError::Unsupported(Box::from("clipboard operation"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_success_covers_read_written_cleared() {
        assert!(ClipboardResult::Read(DataOffer::text("x")).is_success());
        assert!(ClipboardResult::Written.is_success());
        assert!(ClipboardResult::Cleared.is_success());
        assert!(!ClipboardResult::Empty.is_success());
        assert!(!ClipboardResult::Denied.is_success());
        assert!(!ClipboardResult::Unsupported.is_success());
    }

    #[test]
    fn offer_returns_read_payload_only() {
        let offer = DataOffer::text("hi");
        assert_eq!(ClipboardResult::Read(offer.clone()).offer(), Some(&offer));
        assert_eq!(ClipboardResult::Written.offer(), None);
        assert_eq!(ClipboardResult::Empty.offer(), None);
    }

    #[test]
    fn into_error_maps_non_success_variants() {
        assert_eq!(
            ClipboardResult::Empty.into_error(),
            Some(TransferError::Empty)
        );
        assert_eq!(
            ClipboardResult::Denied.into_error(),
            Some(TransferError::PermissionDenied)
        );
        assert_eq!(
            ClipboardResult::Unsupported.into_error(),
            Some(TransferError::Unsupported(Box::from("clipboard operation")))
        );
    }

    #[test]
    fn into_error_is_none_for_success() {
        assert_eq!(
            ClipboardResult::Read(DataOffer::text("x")).into_error(),
            None
        );
        assert_eq!(ClipboardResult::Written.into_error(), None);
        assert_eq!(ClipboardResult::Cleared.into_error(), None);
    }
}
