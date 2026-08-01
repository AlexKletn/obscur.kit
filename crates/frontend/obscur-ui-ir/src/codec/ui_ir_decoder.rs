#![doc = include_str!("ui_ir_decoder.doc/_module.md")]

use obscur_capability::CapabilityId;

use crate::codec::ui_ir_encoder::{FORMAT_VERSION, MAGIC};
use crate::document::{UiIrDocument, UiIrEvent, UiIrExpression, UiIrNode, UiIrProperty, UiIrValue};
use crate::version::{UiIrVersion, UiIrVersionRange};

#[doc = include_str!("ui_ir_decoder.doc/UiIrDecodeError.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiIrDecodeError {
    #[doc = include_str!("ui_ir_decoder.doc/BadMagic.md")]
    BadMagic,
    #[doc = include_str!("ui_ir_decoder.doc/UnexpectedEof.md")]
    UnexpectedEof,
    #[doc = include_str!("ui_ir_decoder.doc/UnsupportedFormatVersion.md")]
    UnsupportedFormatVersion(u16),
    #[doc = include_str!("ui_ir_decoder.doc/UnsupportedIrVersion.md")]
    UnsupportedIrVersion {
        #[doc = include_str!("ui_ir_decoder.doc/document.md")]
        document: UiIrVersion,
        #[doc = include_str!("ui_ir_decoder.doc/supported.md")]
        supported: UiIrVersionRange,
    },
    #[doc = include_str!("ui_ir_decoder.doc/UnknownNodeKind.md")]
    UnknownNodeKind(u8),
    #[doc = include_str!("ui_ir_decoder.doc/UnknownExpressionTag.md")]
    UnknownExpressionTag(u8),
    #[doc = include_str!("ui_ir_decoder.doc/UnknownValueTag.md")]
    UnknownValueTag(u8),
    #[doc = include_str!("ui_ir_decoder.doc/InvalidUtf8.md")]
    InvalidUtf8,
    #[doc = include_str!("ui_ir_decoder.doc/UnsupportedNodeKind.md")]
    UnsupportedNodeKind(u8),
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
            Self::UnsupportedNodeKind(tag) => {
                write!(f, "unsupported node kind (no constructor): {tag}")
            }
        }
    }
}

impl std::error::Error for UiIrDecodeError {}

#[doc = include_str!("ui_ir_decoder.doc/UiIrDecoder.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct UiIrDecoder;

impl UiIrDecoder {
    #[doc = include_str!("ui_ir_decoder.doc/SUPPORTED_VERSIONS.md")]
    pub const SUPPORTED_VERSIONS: UiIrVersionRange =
        UiIrVersionRange::new(UiIrVersion::new(1, 0), UiIrVersion::CURRENT);

    #[doc = include_str!("ui_ir_decoder.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("ui_ir_decoder.doc/decode.md")]
    pub fn decode(&self, bytes: &[u8]) -> Result<UiIrDocument, UiIrDecodeError> {
        let mut cursor = Cursor::new(bytes);

        let magic = cursor.take(MAGIC.len())?;
        if magic != MAGIC {
            return Err(UiIrDecodeError::BadMagic);
        }
        let format = cursor.read_u16()?;
        if format != FORMAT_VERSION {
            return Err(UiIrDecodeError::UnsupportedFormatVersion(format));
        }

        let major = cursor.read_u16()?;
        let minor = cursor.read_u16()?;
        let version = UiIrVersion::new(major, minor);
        if !Self::SUPPORTED_VERSIONS.contains(version) {
            return Err(UiIrDecodeError::UnsupportedIrVersion {
                document: version,
                supported: Self::SUPPORTED_VERSIONS,
            });
        }

        let cap_count = cursor.read_u32()?;
        let mut caps = Vec::new();
        for _ in 0..cap_count {
            caps.push(cursor.read_str()?.to_owned());
        }

        let root = read_node(&mut cursor)?;

        let mut doc = UiIrDocument::new(root).with_version(version);
        for cap in caps {
            doc = doc.requiring(CapabilityId::new(cap));
        }
        Ok(doc)
    }
}

#[doc = include_str!("ui_ir_decoder.doc/read_node.md")]
fn read_node(cursor: &mut Cursor<'_>) -> Result<UiIrNode, UiIrDecodeError> {
    let id = cursor.read_u64()?;
    let kind = cursor.read_u8()?;
    let tag = cursor.read_str()?.to_owned();
    let content = cursor.read_str()?.to_owned();

    let mut node = match kind {
        0 => UiIrNode::element(id, tag),
        1 => UiIrNode::text(id, content),
        2 => UiIrNode::component(id, tag),
        3 => UiIrNode::fragment(id),
        4 | 5 => return Err(UiIrDecodeError::UnsupportedNodeKind(kind)),
        other => return Err(UiIrDecodeError::UnknownNodeKind(other)),
    };

    let prop_count = cursor.read_u32()?;
    for _ in 0..prop_count {
        let name = cursor.read_str()?.to_owned();
        let value = read_expr(cursor)?;
        node = node.with_property(UiIrProperty::new(name, value));
    }

    let event_count = cursor.read_u32()?;
    for _ in 0..event_count {
        let event = cursor.read_str()?.to_owned();
        let handler = cursor.read_str()?.to_owned();
        node = node.with_event(UiIrEvent::new(event, handler));
    }

    let child_count = cursor.read_u32()?;
    for _ in 0..child_count {
        node = node.with_child(read_node(cursor)?);
    }

    Ok(node)
}

#[doc = include_str!("ui_ir_decoder.doc/read_expr.md")]
fn read_expr(cursor: &mut Cursor<'_>) -> Result<UiIrExpression, UiIrDecodeError> {
    let tag = cursor.read_u8()?;
    match tag {
        0 => Ok(UiIrExpression::literal(read_value(cursor)?)),
        1 => Ok(UiIrExpression::binding(cursor.read_str()?.to_owned())),
        2 => {
            let name = cursor.read_str()?.to_owned();
            let arg_count = cursor.read_u32()?;
            let mut args = Vec::new();
            for _ in 0..arg_count {
                args.push(read_expr(cursor)?);
            }
            Ok(UiIrExpression::call(name, args))
        }
        other => Err(UiIrDecodeError::UnknownExpressionTag(other)),
    }
}

#[doc = include_str!("ui_ir_decoder.doc/read_value.md")]
fn read_value(cursor: &mut Cursor<'_>) -> Result<UiIrValue, UiIrDecodeError> {
    let tag = cursor.read_u8()?;
    match tag {
        0 => Ok(UiIrValue::Null),
        1 => Ok(UiIrValue::Bool(cursor.read_u8()? != 0)),
        2 => Ok(UiIrValue::Int(cursor.read_i64()?)),
        3 => Ok(UiIrValue::Float(cursor.read_f64()?)),
        4 => Ok(UiIrValue::Text(cursor.read_str()?.into())),
        other => Err(UiIrDecodeError::UnknownValueTag(other)),
    }
}

#[doc = include_str!("ui_ir_decoder.doc/Cursor.md")]
struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    #[doc = include_str!("ui_ir_decoder.doc/new_2.md")]
    const fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[doc = include_str!("ui_ir_decoder.doc/take.md")]
    fn take(&mut self, n: usize) -> Result<&'a [u8], UiIrDecodeError> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or(UiIrDecodeError::UnexpectedEof)?;
        let slice = self
            .data
            .get(self.pos..end)
            .ok_or(UiIrDecodeError::UnexpectedEof)?;
        self.pos = end;
        Ok(slice)
    }

    #[doc = include_str!("ui_ir_decoder.doc/read_u8.md")]
    fn read_u8(&mut self) -> Result<u8, UiIrDecodeError> {
        let bytes = self.take(1)?;
        Ok(bytes[0])
    }

    #[doc = include_str!("ui_ir_decoder.doc/read_u16.md")]
    fn read_u16(&mut self) -> Result<u16, UiIrDecodeError> {
        let bytes = self.take(2)?;
        let array = <[u8; 2]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(u16::from_le_bytes(array))
    }

    #[doc = include_str!("ui_ir_decoder.doc/read_u32.md")]
    fn read_u32(&mut self) -> Result<u32, UiIrDecodeError> {
        let bytes = self.take(4)?;
        let array = <[u8; 4]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(u32::from_le_bytes(array))
    }

    #[doc = include_str!("ui_ir_decoder.doc/read_u64.md")]
    fn read_u64(&mut self) -> Result<u64, UiIrDecodeError> {
        let bytes = self.take(8)?;
        let array = <[u8; 8]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(u64::from_le_bytes(array))
    }

    #[doc = include_str!("ui_ir_decoder.doc/read_i64.md")]
    fn read_i64(&mut self) -> Result<i64, UiIrDecodeError> {
        let bytes = self.take(8)?;
        let array = <[u8; 8]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(i64::from_le_bytes(array))
    }

    #[doc = include_str!("ui_ir_decoder.doc/read_f64.md")]
    fn read_f64(&mut self) -> Result<f64, UiIrDecodeError> {
        let bytes = self.take(8)?;
        let array = <[u8; 8]>::try_from(bytes).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        Ok(f64::from_le_bytes(array))
    }

    #[doc = include_str!("ui_ir_decoder.doc/read_str.md")]
    fn read_str(&mut self) -> Result<&'a str, UiIrDecodeError> {
        let len = self.read_u32()?;
        let n = usize::try_from(len).map_err(|_| UiIrDecodeError::UnexpectedEof)?;
        let bytes = self.take(n)?;
        std::str::from_utf8(bytes).map_err(|_| UiIrDecodeError::InvalidUtf8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::UiIrEncoder;

    /// Builds a representative document exercising most of the format.
    fn sample_doc() -> UiIrDocument {
        let root = UiIrNode::element(1, "div")
            .with_property(UiIrProperty::new("class", UiIrExpression::text("card")))
            .with_property(UiIrProperty::new(
                "count",
                UiIrExpression::literal(UiIrValue::Int(3)),
            ))
            .with_event(UiIrEvent::new("click", "submit"))
            .with_child(UiIrNode::text(2, "hello"))
            .with_child(
                UiIrNode::component(3, "Button").with_property(UiIrProperty::new(
                    "on",
                    UiIrExpression::binding("state.open"),
                )),
            );
        UiIrDocument::new(root)
            .with_version(UiIrVersion::new(1, 0))
            .requiring(CapabilityId::new("net"))
    }

    #[test]
    fn full_round_trip_is_equal() {
        let doc = sample_doc();
        let encoder = UiIrEncoder::new();
        let decoder = UiIrDecoder::new();
        assert_eq!(decoder.decode(&encoder.encode(&doc)).unwrap(), doc);
    }

    #[test]
    fn float_value_round_trips_exactly() {
        let doc = UiIrDocument::new(UiIrNode::element(1, "x").with_property(UiIrProperty::new(
            "f",
            UiIrExpression::literal(UiIrValue::Float(1.5)),
        )));
        let bytes = UiIrEncoder::new().encode(&doc);
        let back = UiIrDecoder::new().decode(&bytes).unwrap();
        assert_eq!(back, doc);
        assert_eq!(
            back.root().property("f"),
            Some(&UiIrExpression::literal(UiIrValue::Float(1.5)))
        );
    }

    #[test]
    fn nested_call_expression_round_trips() {
        let expr = UiIrExpression::call(
            "fmt",
            vec![
                UiIrExpression::text("hi"),
                UiIrExpression::call("upper", vec![UiIrExpression::binding("name")]),
            ],
        );
        let doc = UiIrDocument::new(
            UiIrNode::element(1, "span").with_property(UiIrProperty::new("v", expr)),
        );
        let bytes = UiIrEncoder::new().encode(&doc);
        assert_eq!(UiIrDecoder::new().decode(&bytes).unwrap(), doc);
    }

    #[test]
    fn empty_document_round_trips() {
        let doc = UiIrDocument::new(UiIrNode::fragment(0));
        let bytes = UiIrEncoder::new().encode(&doc);
        assert_eq!(UiIrDecoder::new().decode(&bytes).unwrap(), doc);
    }

    #[test]
    fn truncated_bytes_report_eof() {
        let doc = sample_doc();
        let bytes = UiIrEncoder::new().encode(&doc);
        let truncated = &bytes[..bytes.len() - 4];
        assert_eq!(
            UiIrDecoder::new().decode(truncated),
            Err(UiIrDecodeError::UnexpectedEof)
        );
    }

    #[test]
    fn wrong_magic_reports_bad_magic() {
        let bytes = *b"XXXX\x01\x00";
        assert_eq!(
            UiIrDecoder::new().decode(&bytes),
            Err(UiIrDecodeError::BadMagic)
        );
    }

    #[test]
    fn conditional_kind_is_unsupported() {
        // Hand-build a minimal stream whose root node carries kind tag 4.
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"OKIR");
        bytes.extend_from_slice(&1u16.to_le_bytes()); // format version
        bytes.extend_from_slice(&1u16.to_le_bytes()); // major
        bytes.extend_from_slice(&0u16.to_le_bytes()); // minor
        bytes.extend_from_slice(&0u32.to_le_bytes()); // cap count
        bytes.extend_from_slice(&7u64.to_le_bytes()); // node id
        bytes.push(4); // kind = Conditional
        bytes.extend_from_slice(&0u32.to_le_bytes()); // tag (empty)
        bytes.extend_from_slice(&0u32.to_le_bytes()); // content (empty)
        assert_eq!(
            UiIrDecoder::new().decode(&bytes),
            Err(UiIrDecodeError::UnsupportedNodeKind(4))
        );
    }

    #[test]
    fn determinism_encode_is_byte_identical() {
        let doc = sample_doc();
        let encoder = UiIrEncoder::new();
        assert_eq!(encoder.encode(&doc), encoder.encode(&doc));
    }

    #[test]
    fn error_types_display_and_are_std_errors() {
        let err: &dyn std::error::Error = &UiIrDecodeError::BadMagic;
        assert_eq!(err.to_string(), "bad magic: not an OKIR stream");
        assert_eq!(
            UiIrDecodeError::UnsupportedFormatVersion(2).to_string(),
            "unsupported format version: 2"
        );
    }
}
