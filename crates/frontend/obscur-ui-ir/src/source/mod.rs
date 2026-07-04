//! Source maps: [`SourceLocation`] positions, [`SourceReference`] spans, and
//! the [`SourceMap`] linking IR node ids back to their source (spec §2.5).
mod source_location;
pub use source_location::SourceLocation;
mod source_map;
pub use source_map::SourceMap;
mod source_reference;
pub use source_reference::SourceReference;
