#![doc = include_str!("mod.doc/_module.md")]
mod font_decoder;
pub use font_decoder::FontDecoder;
mod image_decoder;
pub use image_decoder::ImageDecoder;
mod svg_decoder;
pub use svg_decoder::SvgDecoder;
mod svg_vectorizer;
pub use svg_vectorizer::SvgVectorizer;
