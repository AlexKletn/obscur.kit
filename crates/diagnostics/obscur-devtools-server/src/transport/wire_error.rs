use core::fmt;

use obscur_devtools_protocol::error::ProtocolError;

#[doc = include_str!("../../docs/api/transport/wire_codec.doc/WireError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    /// The buffer ended before a value that was expected could be read.
    UnexpectedEof,
    /// A length-prefixed string held bytes that were not valid UTF-8.
    InvalidUtf8,
    /// A discriminant tag fell outside the range known for its position; the
    /// payload is the unknown tag.
    UnknownTag(u8),
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "unexpected end of wire buffer"),
            Self::InvalidUtf8 => write!(f, "invalid UTF-8 in wire string"),
            Self::UnknownTag(tag) => write!(f, "unknown wire discriminant tag: {tag}"),
        }
    }
}

impl core::error::Error for WireError {}

impl From<WireError> for ProtocolError {
    fn from(_: WireError) -> Self {
        Self::MalformedMessage
    }
}
