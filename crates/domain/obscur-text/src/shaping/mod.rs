//! Shaping module.
mod glyph_run;
pub use glyph_run::GlyphRun;
mod shaped_glyph;
pub use shaped_glyph::ShapedGlyph;
mod shaping_request;
pub use shaping_request::ShapingRequest;
mod text_direction;
pub use text_direction::TextDirection;
mod text_shaper;
pub use text_shaper::{MockTextShaper, TextShaper};
