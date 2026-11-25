//! What an inline-style compiler answered: the declarations it recovered, and the
//! faults it recovered them *past*.

use obscur_diagnostics::Diagnostic;
use obscur_style::matching::InlineStyle;

#[doc = include_str!("../../docs/api/orchestration/inline_style_outcome.doc/InlineStyleOutcome.md")]
#[derive(Debug, Clone, Default)]
pub struct InlineStyleOutcome {
    /// The declarations that parsed. Empty is a legitimate answer — for an empty
    /// attribute, and for one whose every declaration was dropped.
    style: InlineStyle,
    /// The diagnostics the compilation raised, already carrying the front end's own
    /// codes and severities. The runtime forwards them; it does not classify them,
    /// because the CSS grammar they are about lives a tier away from it.
    faults: Vec<Diagnostic>,
}

impl InlineStyleOutcome {
    /// A clean compilation: the block, and nothing to report.
    #[must_use]
    pub fn new(style: InlineStyle) -> Self {
        Self {
            style,
            faults: Vec::new(),
        }
    }

    /// Attaches the diagnostics raised while compiling this block.
    #[must_use]
    pub fn with_faults(mut self, faults: Vec<Diagnostic>) -> Self {
        self.faults = faults;
        self
    }

    /// Takes both halves. Consuming rather than borrowing because the caller caches the
    /// block and emits the faults once — neither is read from this value again.
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
