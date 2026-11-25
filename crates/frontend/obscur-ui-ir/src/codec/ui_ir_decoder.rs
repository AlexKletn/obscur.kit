#![doc = include_str!("../../docs/api/codec/ui_ir_decoder.doc/_module.md")]

mod decode_cursor;

use decode_cursor::DecodeCursor;
use obscur_capability::CapabilityId;

use crate::codec::ui_ir_encoder::{FORMAT_VERSION, MAGIC};
use crate::document::{UiIrDocument, UiIrEvent, UiIrExpression, UiIrNode, UiIrProperty, UiIrValue};
use crate::version::{UiIrVersion, UiIrVersionRange};

use super::UiIrDecodeError;

#[doc = include_str!("../../docs/api/codec/ui_ir_decoder.doc/UiIrDecoder.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct UiIrDecoder;

impl UiIrDecoder {
    /// The inclusive range of IR schema versions this decoder can read: from
    /// the first shipped version up to [`UiIrVersion::CURRENT`] (spec §2.15). A
    /// document declaring a version outside this range is refused with
    /// [`UiIrDecodeError::UnsupportedIrVersion`].
    pub const SUPPORTED_VERSIONS: UiIrVersionRange =
        UiIrVersionRange::new(UiIrVersion::new(1, 0), UiIrVersion::CURRENT);

    /// Creates a new decoder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("../../docs/api/codec/ui_ir_decoder.doc/decode.md")]
    pub fn decode(&self, bytes: &[u8]) -> Result<UiIrDocument, UiIrDecodeError> {
        Self::decode_with(bytes, None)
    }

    /// Decodes untrusted UI IR with a node budget and a maximum recursion depth of 64.
    /// Requires the document to consume the entire input.
    ///
    /// # Errors
    ///
    /// Returns the same decoding errors as [`Self::decode`], plus
    /// [`UiIrDecodeError::LimitExceeded`] for an exhausted node or depth budget
    /// and [`UiIrDecodeError::TrailingBytes`] for extra bytes after the document.
    pub fn decode_bounded(
        &self,
        bytes: &[u8],
        max_nodes: usize,
    ) -> Result<UiIrDocument, UiIrDecodeError> {
        Self::decode_with(bytes, Some(max_nodes))
    }

    fn decode_with(
        bytes: &[u8],
        max_nodes: Option<usize>,
    ) -> Result<UiIrDocument, UiIrDecodeError> {
        let mut cursor = DecodeCursor::new(bytes, max_nodes);

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

        let root = read_node(&mut cursor, 0)?;
        if max_nodes.is_some() && !cursor.is_at_end() {
            return Err(UiIrDecodeError::TrailingBytes);
        }

        let mut doc = UiIrDocument::new(root).with_version(version);
        for cap in caps {
            doc = doc.requiring(CapabilityId::new(cap));
        }
        Ok(doc)
    }
}

/// Reads a node and its whole subtree from `cursor`.
fn read_node(cursor: &mut DecodeCursor<'_>, depth: usize) -> Result<UiIrNode, UiIrDecodeError> {
    cursor.consume_node(depth)?;
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
        let value = read_expr(cursor, 0)?;
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
        node = node.with_child(read_node(cursor, depth + 1)?);
    }

    Ok(node)
}

/// Reads an expression and, recursively, any argument expressions.
fn read_expr(
    cursor: &mut DecodeCursor<'_>,
    depth: usize,
) -> Result<UiIrExpression, UiIrDecodeError> {
    cursor.check_depth(depth)?;
    let tag = cursor.read_u8()?;
    match tag {
        0 => Ok(UiIrExpression::literal(read_value(cursor)?)),
        1 => Ok(UiIrExpression::binding(cursor.read_str()?.to_owned())),
        2 => {
            let name = cursor.read_str()?.to_owned();
            let arg_count = cursor.read_u32()?;
            let mut args = Vec::new();
            for _ in 0..arg_count {
                args.push(read_expr(cursor, depth + 1)?);
            }
            Ok(UiIrExpression::call(name, args))
        }
        other => Err(UiIrDecodeError::UnknownExpressionTag(other)),
    }
}

/// Reads a literal value from `cursor`.
fn read_value(cursor: &mut DecodeCursor<'_>) -> Result<UiIrValue, UiIrDecodeError> {
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
