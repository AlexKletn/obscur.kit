//! Concrete resource decoders, one per format family (spec §5.15).
mod font_decoder;
pub use font_decoder::FontDecoder;
mod image_decoder;
pub use image_decoder::ImageDecoder;
mod svg_decoder;
pub use svg_decoder::SvgDecoder;
