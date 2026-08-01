#![doc = include_str!("mod.doc/_module.md")]

mod display_converter;
pub use display_converter::{DisplayConverter, MonitorSnapshot};
mod keyboard_converter;
pub use keyboard_converter::KeyboardConverter;
mod pointer_converter;
pub use pointer_converter::PointerConverter;
mod window_event_converter;
pub use window_event_converter::WindowEventConverter;
