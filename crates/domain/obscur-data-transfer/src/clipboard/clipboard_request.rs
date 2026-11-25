//! A copy/cut/paste/clear command addressed to the clipboard (spec §6.20 data
//! transfer — clipboard).

use crate::offer::MimeType;
use crate::transfer::TransferPayload;

use super::ClipboardWriteMode;

#[doc = include_str!("../../docs/api/clipboard/clipboard_request.doc/ClipboardRequest.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardRequest {
    /// Copy or cut a payload onto the clipboard (spec §6.20 COPY/CUT).
    Write {
        /// The content to place on the clipboard.
        payload: TransferPayload,
        /// Whether this is a copy or a cut.
        mode: ClipboardWriteMode,
    },
    /// Paste from the clipboard, requesting these formats (empty = any/all
    /// available) (spec §6.20 PASTE).
    Read {
        /// The requested formats; an empty list accepts any available format.
        mimes: Vec<MimeType>,
    },
    /// Empty the clipboard (spec §6.20).
    Clear,
}

impl ClipboardRequest {
    /// Creates a copy request placing `payload` on the clipboard (spec §6.20
    /// COPY).
    #[must_use]
    pub const fn copy(payload: TransferPayload) -> Self {
        Self::Write {
            payload,
            mode: ClipboardWriteMode::Copy,
        }
    }

    /// Creates a cut request placing `payload` on the clipboard (spec §6.20
    /// CUT).
    #[must_use]
    pub const fn cut(payload: TransferPayload) -> Self {
        Self::Write {
            payload,
            mode: ClipboardWriteMode::Cut,
        }
    }

    /// Creates a paste request narrowed to `mimes` (empty = any/all available)
    /// (spec §6.20 PASTE).
    #[must_use]
    pub const fn read(mimes: Vec<MimeType>) -> Self {
        Self::Read { mimes }
    }

    /// Creates a paste request accepting any available format (spec §6.20
    /// PASTE).
    #[must_use]
    pub const fn read_any() -> Self {
        Self::Read { mimes: Vec::new() }
    }

    /// Creates a clear request (spec §6.20).
    #[must_use]
    pub const fn clear() -> Self {
        Self::Clear
    }

    /// Returns whether this is a [`Write`](ClipboardRequest::Write) (copy or
    /// cut) (spec §6.20).
    #[must_use]
    pub const fn is_write(&self) -> bool {
        matches!(self, Self::Write { .. })
    }

    /// Returns whether this is a [`Read`](ClipboardRequest::Read) (paste)
    /// (spec §6.20).
    #[must_use]
    pub const fn is_read(&self) -> bool {
        matches!(self, Self::Read { .. })
    }

    /// Returns the [`ClipboardWriteMode`] of a write request, or `None` for a
    /// read or clear (spec §6.20).
    #[must_use]
    pub const fn write_mode(&self) -> Option<ClipboardWriteMode> {
        match self {
            Self::Write { mode, .. } => Some(*mode),
            Self::Read { .. } | Self::Clear => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offer::DataOffer;

    fn payload() -> TransferPayload {
        TransferPayload::new(DataOffer::text("hi"))
    }

    #[test]
    fn copy_and_cut_carry_their_mode() {
        let copy = ClipboardRequest::copy(payload());
        let cut = ClipboardRequest::cut(payload());
        assert_eq!(copy.write_mode(), Some(ClipboardWriteMode::Copy));
        assert_eq!(cut.write_mode(), Some(ClipboardWriteMode::Cut));
        assert!(copy.is_write());
        assert!(cut.is_write());
        assert!(!copy.is_read());
    }

    #[test]
    fn read_carries_requested_mimes() {
        let req = ClipboardRequest::read(vec![MimeType::text_plain()]);
        assert!(req.is_read());
        assert!(!req.is_write());
        assert_eq!(req.write_mode(), None);
        match req {
            ClipboardRequest::Read { mimes } => {
                assert_eq!(mimes, vec![MimeType::text_plain()]);
            }
            _ => panic!("expected read"),
        }
    }

    #[test]
    fn read_any_requests_no_specific_format() {
        match ClipboardRequest::read_any() {
            ClipboardRequest::Read { mimes } => assert!(mimes.is_empty()),
            _ => panic!("expected read"),
        }
    }

    #[test]
    fn clear_is_neither_read_nor_write() {
        let req = ClipboardRequest::clear();
        assert_eq!(req, ClipboardRequest::Clear);
        assert!(!req.is_read());
        assert!(!req.is_write());
        assert_eq!(req.write_mode(), None);
    }
}
