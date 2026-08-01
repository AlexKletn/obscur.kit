#![doc = include_str!("clipboard_request.doc/_module.md")]

use crate::offer::MimeType;
use crate::transfer::TransferPayload;

#[doc = include_str!("clipboard_request.doc/ClipboardWriteMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardWriteMode {
    #[doc = include_str!("clipboard_request.doc/Copy.md")]
    Copy,
    #[doc = include_str!("clipboard_request.doc/Cut.md")]
    Cut,
}

impl ClipboardWriteMode {
    #[doc = include_str!("clipboard_request.doc/is_cut.md")]
    #[must_use]
    pub const fn is_cut(self) -> bool {
        matches!(self, Self::Cut)
    }
}

#[doc = include_str!("clipboard_request.doc/ClipboardRequest.md")]
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardRequest {
    #[doc = include_str!("clipboard_request.doc/Write.md")]
    Write {
        #[doc = include_str!("clipboard_request.doc/payload.md")]
        payload: TransferPayload,
        #[doc = include_str!("clipboard_request.doc/mode.md")]
        mode: ClipboardWriteMode,
    },
    #[doc = include_str!("clipboard_request.doc/Read.md")]
    Read {
        #[doc = include_str!("clipboard_request.doc/mimes.md")]
        mimes: Vec<MimeType>,
    },
    #[doc = include_str!("clipboard_request.doc/Clear.md")]
    Clear,
}

impl ClipboardRequest {
    #[doc = include_str!("clipboard_request.doc/copy.md")]
    #[must_use]
    pub const fn copy(payload: TransferPayload) -> Self {
        Self::Write {
            payload,
            mode: ClipboardWriteMode::Copy,
        }
    }

    #[doc = include_str!("clipboard_request.doc/cut.md")]
    #[must_use]
    pub const fn cut(payload: TransferPayload) -> Self {
        Self::Write {
            payload,
            mode: ClipboardWriteMode::Cut,
        }
    }

    #[doc = include_str!("clipboard_request.doc/read.md")]
    #[must_use]
    pub const fn read(mimes: Vec<MimeType>) -> Self {
        Self::Read { mimes }
    }

    #[doc = include_str!("clipboard_request.doc/read_any.md")]
    #[must_use]
    pub const fn read_any() -> Self {
        Self::Read { mimes: Vec::new() }
    }

    #[doc = include_str!("clipboard_request.doc/clear.md")]
    #[must_use]
    pub const fn clear() -> Self {
        Self::Clear
    }

    #[doc = include_str!("clipboard_request.doc/is_write.md")]
    #[must_use]
    pub const fn is_write(&self) -> bool {
        matches!(self, Self::Write { .. })
    }

    #[doc = include_str!("clipboard_request.doc/is_read.md")]
    #[must_use]
    pub const fn is_read(&self) -> bool {
        matches!(self, Self::Read { .. })
    }

    #[doc = include_str!("clipboard_request.doc/write_mode.md")]
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
