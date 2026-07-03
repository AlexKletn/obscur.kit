//! A copy/cut/paste/clear command addressed to the clipboard (spec §6.20 data
//! transfer — clipboard).

use crate::offer::MimeType;
use crate::transfer::TransferPayload;

/// How a write places content on the clipboard (spec §6.20 COPY/CUT).
///
/// The two modes store identical bytes; they differ only in what the *source*
/// does afterward. [`Copy`](ClipboardWriteMode::Copy) leaves the source content
/// in place, while [`Cut`](ClipboardWriteMode::Cut) asks the source to delete
/// its content once the clipboard has been read (a move). Enforcing the deletion
/// is the source's concern — this value only records the intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardWriteMode {
    /// Copy: the source keeps its content (spec §6.20 COPY).
    Copy,
    /// Cut: the source should delete its content after a successful read
    /// (spec §6.20 CUT).
    Cut,
}

impl ClipboardWriteMode {
    /// Returns whether this is [`ClipboardWriteMode::Cut`] (spec §6.20 CUT).
    #[must_use]
    pub const fn is_cut(self) -> bool {
        matches!(self, Self::Cut)
    }
}

/// A single clipboard command: copy/cut *to* the clipboard, paste *from* it, or
/// clear it (spec §6.20 data transfer — clipboard).
///
/// The clipboard and drag-and-drop share the data-transfer subsystem; this type
/// models the clipboard's three verbs as one value driven through the
/// [`ClipboardProvider`](super::ClipboardProvider) port. A
/// [`Write`](ClipboardRequest::Write) carries the [`TransferPayload`] to store
/// and its [`ClipboardWriteMode`]; a [`Read`](ClipboardRequest::Read) requests a
/// paste, optionally narrowing to specific [`MimeType`]s (an empty list means
/// "any/all available"); [`Clear`](ClipboardRequest::Clear) empties the
/// clipboard. The command is a pure, deterministic value carrying no I/O.
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
    fn write_mode_predicates_classify_cut() {
        assert!(ClipboardWriteMode::Cut.is_cut());
        assert!(!ClipboardWriteMode::Copy.is_cut());
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
