#![doc = include_str!("ui_ir_encoder.doc/_module.md")]

use crate::document::{UiIrDocument, UiIrExpression, UiIrNode, UiIrValue};

#[doc = include_str!("ui_ir_encoder.doc/MAGIC.md")]
pub(crate) const MAGIC: [u8; 4] = *b"OKIR";
#[doc = include_str!("ui_ir_encoder.doc/FORMAT_VERSION.md")]
pub(crate) const FORMAT_VERSION: u16 = 1;

#[doc = include_str!("ui_ir_encoder.doc/UiIrEncoder.md")]
#[derive(Debug, Default, Clone, Copy)]
pub struct UiIrEncoder;

impl UiIrEncoder {
    #[doc = include_str!("ui_ir_encoder.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[doc = include_str!("ui_ir_encoder.doc/encode.md")]
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

#[doc = include_str!("ui_ir_encoder.doc/len_u32.md")]
fn len_u32(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

#[doc = include_str!("ui_ir_encoder.doc/push_u8.md")]
fn push_u8(buf: &mut Vec<u8>, value: u8) {
    buf.push(value);
}

#[doc = include_str!("ui_ir_encoder.doc/push_u16.md")]
fn push_u16(buf: &mut Vec<u8>, value: u16) {
    buf.extend_from_slice(&value.to_le_bytes());
}

#[doc = include_str!("ui_ir_encoder.doc/push_u32.md")]
fn push_u32(buf: &mut Vec<u8>, value: u32) {
    buf.extend_from_slice(&value.to_le_bytes());
}

#[doc = include_str!("ui_ir_encoder.doc/push_u64.md")]
fn push_u64(buf: &mut Vec<u8>, value: u64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

#[doc = include_str!("ui_ir_encoder.doc/push_i64.md")]
fn push_i64(buf: &mut Vec<u8>, value: i64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

#[doc = include_str!("ui_ir_encoder.doc/push_f64.md")]
fn push_f64(buf: &mut Vec<u8>, value: f64) {
    buf.extend_from_slice(&value.to_le_bytes());
}

#[doc = include_str!("ui_ir_encoder.doc/push_str.md")]
fn push_str(buf: &mut Vec<u8>, value: &str) {
    let bytes = value.as_bytes();
    push_u32(buf, len_u32(bytes.len()));
    buf.extend_from_slice(bytes);
}

#[doc = include_str!("ui_ir_encoder.doc/kind_tag.md")]
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

#[doc = include_str!("ui_ir_encoder.doc/push_node.md")]
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

#[doc = include_str!("ui_ir_encoder.doc/push_expr.md")]
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

#[doc = include_str!("ui_ir_encoder.doc/push_value.md")]
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
        let doc = UiIrDocument::new(UiIrNode::element(1, "div").with_property(
            crate::document::UiIrProperty::new("class", UiIrExpression::text("card")),
        ));
        let encoder = UiIrEncoder::new();
        assert_eq!(encoder.encode(&doc), encoder.encode(&doc));
    }

    #[test]
    fn len_u32_saturates() {
        assert_eq!(len_u32(3), 3);
        assert_eq!(len_u32(usize::MAX), u32::MAX);
    }
}
