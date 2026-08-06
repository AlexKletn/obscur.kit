#![doc = include_str!("inline_style_outcome.doc/_module.md")]

use obscur_diagnostics::Diagnostic;
use obscur_style::matching::InlineStyle;

#[doc = include_str!("inline_style_outcome.doc/InlineStyleOutcome.md")]
#[derive(Debug, Clone, Default)]
pub struct InlineStyleOutcome {
    #[doc = include_str!("inline_style_outcome.doc/style.md")]
    style: InlineStyle,
    #[doc = include_str!("inline_style_outcome.doc/faults.md")]
    faults: Vec<Diagnostic>,
}

impl InlineStyleOutcome {
    #[doc = include_str!("inline_style_outcome.doc/new.md")]
    #[must_use]
    pub fn new(style: InlineStyle) -> Self {
        Self {
            style,
            faults: Vec::new(),
        }
    }

    #[doc = include_str!("inline_style_outcome.doc/with_faults.md")]
    #[must_use]
    pub fn with_faults(mut self, faults: Vec<Diagnostic>) -> Self {
        self.faults = faults;
        self
    }

    #[doc = include_str!("inline_style_outcome.doc/into_parts.md")]
    #[must_use]
    pub fn into_parts(self) -> (InlineStyle, Vec<Diagnostic>) {
        (self.style, self.faults)
    }
}

#[cfg(test)]
mod tests {
    use super::InlineStyleOutcome;
    use obscur_diagnostics::{Diagnostic, DiagnosticCode, DiagnosticSeverity, DiagnosticSubsystem};
    use obscur_style::matching::{BlockDeclaration, InlineStyle};
    use obscur_style::property::{PropertyId, PropertyValue};

    fn fault() -> Diagnostic {
        Diagnostic::new(
            DiagnosticCode::new("css.invalid_value"),
            DiagnosticSeverity::Warning,
            DiagnosticSubsystem::Style,
            "nope",
        )
    }

    #[test]
    fn a_block_that_parsed_cleanly_reports_nothing() {
        let (style, faults) = InlineStyleOutcome::new(InlineStyle::empty()).into_parts();
        assert!(style.is_empty());
        assert!(faults.is_empty());
    }

    #[test]
    fn what_parsed_and_what_failed_travel_together() {
        // The pair is the whole point of the type: a front end that returned only
        // the block would drop the faults on the floor, and `style="width: nope"`
        // would be indistinguishable from no attribute at all (§1.6).
        let block = InlineStyle::new(vec![BlockDeclaration::normal(
            PropertyId::Opacity,
            PropertyValue::Number(0.5),
        )]);
        let (style, faults) = InlineStyleOutcome::new(block)
            .with_faults(vec![fault()])
            .into_parts();
        assert_eq!(style.declarations().len(), 1, "the rest of the block held");
        assert_eq!(faults.len(), 1);
    }
}
