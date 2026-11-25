//! Backend-independent shaped-text payloads carried by `DrawText`.
mod glyph_run;
pub use glyph_run::GlyphRun;
mod positioned_glyph;
pub use positioned_glyph::PositionedGlyph;

mod synthetic_bold;
pub use synthetic_bold::SYNTHETIC_BOLD_OUTSET_FRACTION;
