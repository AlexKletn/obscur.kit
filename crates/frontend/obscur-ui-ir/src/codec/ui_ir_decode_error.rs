use crate::version::{UiIrVersion, UiIrVersionRange};

#[doc = include_str!("../../docs/api/codec/ui_ir_decoder.doc/UiIrDecodeError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiIrDecodeError {
    /// The stream did not begin with the `b"OKIR"` magic bytes.
    BadMagic,
    /// The stream ended before a value that was expected could be read.
    UnexpectedEof,
    /// The header carried a format version this codec does not support; the
    /// payload is the version that was read.
    UnsupportedFormatVersion(u16),
    /// The document declared an IR schema version outside the range this codec
    /// can read (spec §2.15 "precise diagnostics for newer unsupported IR"). The
    /// payload carries the document's declared version and the codec's supported
    /// range so the diagnostic can name both.
    UnsupportedIrVersion {
        /// The IR version the document declared.
        document: UiIrVersion,
        /// The inclusive range of IR versions this codec can read.
        supported: UiIrVersionRange,
    },
    /// A node carried a kind tag outside the known 0..=5 range; the payload is
    /// the unknown tag.
    UnknownNodeKind(u8),
    /// An expression carried a tag outside the known 0..=2 range; the payload is
    /// the unknown tag.
    UnknownExpressionTag(u8),
    /// A value carried a tag outside the known 0..=4 range; the payload is the
    /// unknown tag.
    UnknownValueTag(u8),
    /// A length-prefixed string held bytes that were not valid UTF-8.
    InvalidUtf8,
    /// A node used a known but not-yet-constructible kind (`Conditional` or
    /// `Loop`); the payload is that kind's tag. See the decoder module documentation.
    UnsupportedNodeKind(u8),
    /// A bounded document exceeded its node or recursion budget.
    LimitExceeded,
    /// The bounded document contains bytes after its root.
    TrailingBytes,
}

impl std::fmt::Display for UiIrDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadMagic => write!(f, "bad magic: not an OKIR stream"),
            Self::UnexpectedEof => write!(f, "unexpected end of input"),
            Self::UnsupportedFormatVersion(v) => {
                write!(f, "unsupported format version: {v}")
            }
            Self::UnsupportedIrVersion {
                document,
                supported,
            } => write!(
                f,
                "unsupported IR version {}.{}: this codec reads IR versions \
                 {}.{} through {}.{}",
                document.major(),
                document.minor(),
                supported.min().major(),
                supported.min().minor(),
                supported.max().major(),
                supported.max().minor(),
            ),
            Self::UnknownNodeKind(tag) => write!(f, "unknown node kind tag: {tag}"),
            Self::UnknownExpressionTag(tag) => {
                write!(f, "unknown expression tag: {tag}")
            }
            Self::UnknownValueTag(tag) => write!(f, "unknown value tag: {tag}"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in string"),
            Self::LimitExceeded => write!(f, "UI IR decode limit exceeded"),
            Self::TrailingBytes => write!(f, "unexpected bytes after UI IR document"),
            Self::UnsupportedNodeKind(tag) => {
                write!(f, "unsupported node kind (no constructor): {tag}")
            }
        }
    }
}

impl std::error::Error for UiIrDecodeError {}
