//! Diagnostics produced while parsing and compiling CSS (spec §4.13).
//!
//! [`CssParseError`] reports syntactic errors, while [`UnsupportedCssFeature`]
//! explicitly surfaces recognised-but-unimplemented constructs so the compiler
//! can degrade gracefully rather than hard-fail. Their categories are the
//! colocated [`CssParseErrorKind`] and [`UnsupportedFeatureKind`] enums.

mod css_parse_error;
pub use css_parse_error::{CssParseError, CssParseErrorKind};
mod unsupported_css_feature;
pub use unsupported_css_feature::{UnsupportedCssFeature, UnsupportedFeatureKind};
