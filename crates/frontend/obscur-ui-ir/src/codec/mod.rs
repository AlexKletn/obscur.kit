#![doc = include_str!("../../docs/api/codec/mod.doc/_module.md")]
mod ui_ir_decoder;
pub use ui_ir_decoder::UiIrDecoder;
mod ui_ir_decode_error;
pub use ui_ir_decode_error::UiIrDecodeError;
mod ui_ir_encoder;
pub use ui_ir_encoder::UiIrEncoder;
