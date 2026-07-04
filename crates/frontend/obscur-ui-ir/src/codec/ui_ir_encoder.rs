//! `UiIrEncoder`: the deterministic binary encoder for [`UiIrDocument`]
//! (spec §2.5 codecs).
//!
//! # Binary format (version 1)
//!
//! All multi-byte integers and floats are **little-endian**, written via
//! `to_le_bytes`. The encoding is total: it never fails.
//!
//! - **Header**: magic `b"OKIR"` (4 bytes), then the format version `u16` = 1.
//! - **Document**: the [`UiIrVersion`] (`major` u16, then `minor` u16); the
//!   `required_capabilities` as a `u32` count followed by each id as a
//!   length-prefixed string; then the root [`UiIrNode`].
//! - **Node**: `id` u64; `kind` u8 (Element=0, Text=1, Component=2, Fragment=3,
//!   Conditional=4, Loop=5); `tag` string; `content` string; `properties`
//!   (`u32` count, each = name string + expression); `events` (`u32` count,
//!   each = event string + handler string); `children` (`u32` count, each a
//!   node, recursive).
//! - **Expression**: a `u8` tag — 0 `Literal` (+ value), 1 `Binding`
//!   (+ string), 2 `Call` (+ name string + `u32` arg count + each arg an
//!   expression, recursive).
//! - **Value**: a `u8` tag — 0 `Null`, 1 `Bool` (+ u8), 2 `Int` (+ i64),
//!   3 `Float` (+ f64), 4 `Text` (+ string).
//! - **String**: a `u32` byte-length then the UTF-8 bytes.
//!
//! The layout is fixed and field order is stable, so encoding is deterministic:
//! `encode(doc)` is byte-identical across runs. Decode it with
//! [`UiIrDecoder`](crate::codec::UiIrDecoder); the round-trip
//! `decode(encode(doc)) == doc` is the contract.

use crate::document::{UiIrDocument, UiIrExpression, UiIrNode, UiIrValue};

/// Magic bytes marking an Obscur Kit UI IR stream.
pub(crate) const MAGIC: [u8; 4] = *b"OKIR";
/// The binary format version this codec reads and writes.
pub(crate) const FORMAT_VERSION: u16 = 1;

/// A stateless, deterministic binary encoder for [`UiIrDocument`] (spec §2.5).
///
/// The encoder serializes a document into the compact little-endian layout
/// documented at the [module level](self). It holds no state — every call is
/// independent — and encoding is total (it never fails). Decode the produced
/// bytes with [`UiIrDecoder`](crate::codec::UiIrDecoder).
#[derive(Debug, Default, Clone, Copy)]
pub struct UiIrEncoder;

impl UiIrEncoder {
    /// Creates a new encoder.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Encodes `doc` into a fresh byte buffer in the format documented at the
    /// [module level](self). Encoding is total and never fails.
    #[must_use]
    pub fn encode(&self, doc: &UiIrDocument) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&MAGIC);
        push_u16(&mut buf, FORMAT_VERSION);

        let version = doc.version();
        push_u16(&mut buf, version.major());
        push_u16(&mut buf, version.minor());

        let caps = doc.required_capabilities();
        push_u32(&mut buf, len_u32(caps.len()));
        for cap in caps {
            push_str(&mut buf, cap.as_str());
        }

        push_node(&mut buf, doc.root());
        buf
    }
}

/// Narrows a `usize` length to `u32`, saturating at `u32::MAX` (no `as` cast).
fn len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

/// Pushes a single byte.
fn push_u8(buf: &mut Vec<u8>, value: u8) {
    buf.push(value);
}

/// Pushes a little-endian `u16`.
fn push_u16(buf: &mut Vec<u8>, value: u16) {
    buf.extend_from_slice(&value.to_le_bytes());
}

/// Pushes a little-endian `u32`.
fn push_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

/// Pushes a little-endian `u64`.
fn push_u64(buf: &mut Vec<u8>, value: u64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

/// Pushes a little-endian `i64`.
fn push_i64(buf: &mut Vec<u8>, value: i64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

/// Pushes a little-endian `f64`.
fn push_f64(buf: &mut Vec<u8>, value: f64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

/// Pushes a string as a `u32` byte-length prefix followed by its UTF-8 bytes.
fn push_str(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    push_u32(buf, len_u32(bytes.len()));
    buf.extend_from_slice(bytes);
}

/// Maps a node kind to its wire tag byte.
fn kind_tag(kind: crate::document::UiIrNodeKind) -> u8 {
    use crate::document::UiIrNodeKind;
    match kind {
        UiIrNodeKind::Element => 0,
        UiIrNodeKind::Text => 1,
        UiIrNodeKind::Component => 2,
        UiIrNodeKind::Fragment => 3,
        UiIrNodeKind::Conditional => 4,
        UiIrNodeKind::Loop => 5,
    }
}

/// Writes a node and, recursively, its whole subtree.
fn push_node(buf: &mut Vec<u8>, node: &UiIrNode) {
    push_u64(buf, node.id());
    push_u8(buf, kind_tag(node.kind()));
    push_str(buf, node.tag());
    push_str(buf, node.content());

    let props = node.properties();
    push_u32(buf, len_u32(props.len()));
    for prop in props {
        push_str(buf, prop.name());
        push_expr(buf, prop.value());
    }

    let events = node.events();
    push_u32(buf, len_u32(events.len()));
    for event in events {
        push_str(buf, event.event());
        push_str(buf, event.handler());
    }

    let children = node.children();
    push_u32(buf, len_u32(children.len()));
    for child in children {
        push_node(buf, child);
    }
}

/// Writes an expression and, recursively, any argument expressions.
fn push_expr(buf: &mut Vec<u8>, expr: &UiIrExpression) {
    match expr {
        UiIrExpression::Literal(value) => {
            push_u8(buf, 0);
            push_value(buf, value);
        }
        UiIrExpression::Binding(path) => {
            push_u8(buf, 1);
            push_str(buf, path);
        }
        UiIrExpression::Call { name, args } => {
            push_u8(buf, 2);
            push_str(buf, name);
            push_u32(buf, len_u32(args.len()));
            for arg in args {
                push_expr(buf, arg);
            }
        }
    }
}

/// Writes a literal value.
fn push_value(buf: &mut Vec<u8>, value: &UiIrValue) {
    match value {
        UiIrValue::Null => push_u8(buf, 0),
        UiIrValue::Bool(b) => {
            push_u8(buf, 1);
            push_u8(buf, u8::from(*b));
        }
        UiIrValue::Int(i) => {
            push_u8(buf, 2);
            push_i64(buf, *i);
        }
        UiIrValue::Float(f) => {
            push_u8(buf, 3);
            push_f64(buf, *f);
        }
        UiIrValue::Text(t) => {
            push_u8(buf, 4);
            push_str(buf, t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_is_magic_then_format_version() {
        let doc = UiIrDocument::new(UiIrNode::fragment(0));
        let bytes = UiIrEncoder::new().encode(&doc);
        assert_eq!(&bytes[0..4], b"OKIR");
        assert_eq!(&bytes[4..6], &FORMAT_VERSION.to_le_bytes());
    }

    #[test]
    fn encoding_is_deterministic() {
        let doc = UiIrDocument::new(
            UiIrNode::element(1, "div").with_property(crate::document::UiIrProperty::new(
                "class",
                UiIrExpression::text("card"),
            )),
        );
        let encoder = UiIrEncoder::new();
        assert_eq!(encoder.encode(&doc), encoder.encode(&doc));
    }

    #[test]
    fn len_u32_saturates() {
        assert_eq!(len_u32(3), 3);
        assert_eq!(len_u32(usize::MAX), u32::MAX);
    }
}
