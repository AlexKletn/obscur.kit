//! A CSS feature the kit does not support, surfaced explicitly (spec §4.13).

use crate::source::SourceSpan;

use super::UnsupportedFeatureKind;

#[doc = include_str!("../../docs/api/error/unsupported_css_feature.doc/UnsupportedCssFeature.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedCssFeature {
    kind: UnsupportedFeatureKind,
    name: Box<str>,
    span: SourceSpan,
}

impl UnsupportedCssFeature {
    /// Creates a diagnostic for an unsupported feature of `kind` named `name`
    /// covering `span`.
    #[must_use]
    pub fn new(kind: UnsupportedFeatureKind, name: impl Into<Box<str>>, span: SourceSpan) -> Self {
        Self {
            kind,
            name: name.into(),
            span,
        }
    }

    /// Returns the category of the unsupported feature.
    #[must_use]
    pub const fn kind(&self) -> UnsupportedFeatureKind {
        self.kind
    }

    /// Returns the offending token (for example `"@supports"`).
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the span the unsupported feature occupies.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
}

impl std::fmt::Display for UnsupportedCssFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} at {}..{}",
            self.kind.label(),
            self.name,
            self.span.start(),
            self.span.end()
        )
    }
}

impl std::error::Error for UnsupportedCssFeature {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_kind_name_and_span() {
        let feat = UnsupportedCssFeature::new(
            UnsupportedFeatureKind::AtRule,
            "@supports",
            SourceSpan::new(0, 9),
        );
        assert_eq!(feat.to_string(), "unsupported at-rule: @supports at 0..9");
    }

    #[test]
    fn accessors_report_fields() {
        let feat = UnsupportedCssFeature::new(
            UnsupportedFeatureKind::Property,
            "grid-template-areas",
            SourceSpan::new(4, 23),
        );
        assert_eq!(feat.kind(), UnsupportedFeatureKind::Property);
        assert_eq!(feat.name(), "grid-template-areas");
        assert_eq!(feat.span(), SourceSpan::new(4, 23));
    }

    #[test]
    fn usable_as_std_error() {
        let feat = UnsupportedCssFeature::new(
            UnsupportedFeatureKind::Selector,
            ":has",
            SourceSpan::new(1, 5),
        );
        let dyn_err: &dyn std::error::Error = &feat;
        assert_eq!(dyn_err.to_string(), "unsupported selector: :has at 1..5");
    }
}
