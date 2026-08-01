#![doc = include_str!("mod.doc/_module.md")]

mod css_source;
pub use css_source::CssSource;
mod source_id;
pub use source_id::SourceId;
mod source_version;
pub use source_version::SourceVersion;
mod source_span;
pub use source_span::SourceSpan;
