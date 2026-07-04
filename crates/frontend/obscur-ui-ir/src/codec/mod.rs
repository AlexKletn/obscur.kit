//! Deterministic binary codecs for [`UiIrDocument`](crate::document::UiIrDocument)
//! (spec §2.5): a hand-rolled little-endian [`UiIrEncoder`]/[`UiIrDecoder`]
//! pair with no serde or external-crate dependency.
//!
//! The two halves are exact inverses — `decode(encode(doc)) == doc` is the
//! contract. The wire format (magic, header, node/expression/value layout) is
//! documented on [`UiIrEncoder`].
mod ui_ir_decoder;
pub use ui_ir_decoder::{UiIrDecodeError, UiIrDecoder};
mod ui_ir_encoder;
pub use ui_ir_encoder::UiIrEncoder;
