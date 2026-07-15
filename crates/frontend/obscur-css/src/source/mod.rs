//! Source identity and byte spans for parsed CSS (spec §2.5 source maps).
//!
//! A [`CssSource`] owns a stylesheet's [`SourceId`], logical name, and text;
//! parsers and diagnostics locate substrings within it using [`SourceSpan`]
//! byte ranges.

mod css_source;
pub use css_source::CssSource;
mod source_id;
pub use source_id::SourceId;
mod source_version;
pub use source_version::SourceVersion;
mod source_span;
pub use source_span::SourceSpan;
