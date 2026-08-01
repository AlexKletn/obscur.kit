#![doc = include_str!("mod.doc/_module.md")]
mod ui_ir_decoder;
pub use ui_ir_decoder::{UiIrDecodeError, UiIrDecoder};
mod ui_ir_encoder;
pub use ui_ir_encoder::UiIrEncoder;
