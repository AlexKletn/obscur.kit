#![doc = include_str!("unsupported_css_feature.doc/_module.md")]

use crate::source::SourceSpan;

#[doc = include_str!("unsupported_css_feature.doc/UnsupportedFeatureKind.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnsupportedFeatureKind {
    #[doc = include_str!("unsupported_css_feature.doc/AtRule.md")]
    AtRule,
    #[doc = include_str!("unsupported_css_feature.doc/Property.md")]
    Property,
    #[doc = include_str!("unsupported_css_feature.doc/Selector.md")]
    Selector,
    #[doc = include_str!("unsupported_css_feature.doc/Function.md")]
    Function,
    #[doc = include_str!("unsupported_css_feature.doc/Unit.md")]
    Unit,
    #[doc = include_str!("unsupported_css_feature.doc/PseudoElement.md")]
    PseudoElement,
}

impl UnsupportedFeatureKind {
    #[doc = include_str!("unsupported_css_feature.doc/label.md")]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AtRule => "unsupported at-rule",
            Self::Property => "unsupported property",
            Self::Selector => "unsupported selector",
            Self::Function => "unsupported function",
            Self::Unit => "unsupported unit",
            Self::PseudoElement => "unsupported pseudo-element",
        }
    }
}

#[doc = include_str!("unsupported_css_feature.doc/UnsupportedCssFeature.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedCssFeature {
    kind: UnsupportedFeatureKind,
    name: Box<str>,
    span: SourceSpan,
}

impl UnsupportedCssFeature {
    #[doc = include_str!("unsupported_css_feature.doc/new.md")]
    #[must_use]
    pub fn new(kind: UnsupportedFeatureKind, name: impl Into<Box<str>>, span: SourceSpan) -> Self {
        Self {
            kind,
            name: name.into(),
            span,
        }
    }

    #[doc = include_str!("unsupported_css_feature.doc/kind.md")]
    #[must_use]
    pub const fn kind(&self) -> UnsupportedFeatureKind {
        self.kind
    }

    #[doc = include_str!("unsupported_css_feature.doc/name.md")]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[doc = include_str!("unsupported_css_feature.doc/span.md")]
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
