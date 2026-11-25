//! A diagnostic sink that writes to stderr.

use obscur_diagnostics::{Diagnostic, DiagnosticSeverity, DiagnosticSink};

#[doc = include_str!("../../docs/api/bootstrap/stderr_diagnostics.doc/StderrDiagnostics.md")]
#[derive(Debug, Clone, Copy)]
pub struct StderrDiagnostics {
    /// The lowest severity that is worth a line.
    floor: DiagnosticSeverity,
}

impl StderrDiagnostics {
    #[doc = include_str!("../../docs/api/bootstrap/stderr_diagnostics.doc/new.md")]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            floor: DiagnosticSeverity::Warning,
        }
    }

    /// Report from a chosen severity upward.
    #[must_use]
    pub const fn from_severity(floor: DiagnosticSeverity) -> Self {
        Self { floor }
    }

    /// Whether a diagnostic of this severity clears the floor.
    #[must_use]
    pub fn reports(&self, severity: DiagnosticSeverity) -> bool {
        rank(severity) >= rank(self.floor)
    }

    #[doc = include_str!("../../docs/api/bootstrap/stderr_diagnostics.doc/format.md")]
    #[must_use]
    pub fn format(diagnostic: &Diagnostic) -> String {
        format!(
            "[obscur {severity}] {subsystem:?}/{code}: {message}",
            severity = label(diagnostic.severity),
            subsystem = diagnostic.subsystem,
            code = diagnostic.code.as_str(),
            message = diagnostic.message,
        )
    }
}

impl Default for StderrDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticSink for StderrDiagnostics {
    fn emit(&self, diagnostic: &Diagnostic) {
        if self.reports(diagnostic.severity) {
            eprintln!("{}", Self::format(diagnostic));
        }
    }
}

/// Severities as comparable numbers, ordered the way they read.
const fn rank(severity: DiagnosticSeverity) -> u8 {
    match severity {
        DiagnosticSeverity::Trace => 0,
        DiagnosticSeverity::Info => 1,
        DiagnosticSeverity::Warning => 2,
        DiagnosticSeverity::Error => 3,
        DiagnosticSeverity::Critical => 4,
    }
}

/// The word that goes in the line.
const fn label(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Trace => "trace",
        DiagnosticSeverity::Info => "info",
        DiagnosticSeverity::Warning => "warn",
        DiagnosticSeverity::Error => "error",
        DiagnosticSeverity::Critical => "critical",
    }
}

#[cfg(test)]
mod tests {
    use obscur_diagnostics::{DiagnosticCode, DiagnosticSubsystem};

    use super::*;

    fn warning() -> Diagnostic {
        Diagnostic::new(
            DiagnosticCode::new("css.unsupported_function"),
            DiagnosticSeverity::Warning,
            DiagnosticSubsystem::Style,
            "unsupported function: background at 2486..2511",
        )
    }

    #[test]
    fn the_default_floor_is_the_one_a_developer_wants_to_see() {
        // Warnings and worse. Trace and info at every frame would drown the
        // thing this exists to make visible.
        let sink = StderrDiagnostics::new();
        assert!(sink.reports(DiagnosticSeverity::Warning));
        assert!(sink.reports(DiagnosticSeverity::Error));
        assert!(sink.reports(DiagnosticSeverity::Critical));
        assert!(!sink.reports(DiagnosticSeverity::Info));
        assert!(!sink.reports(DiagnosticSeverity::Trace));
    }

    #[test]
    fn a_lower_floor_lets_everything_through() {
        let sink = StderrDiagnostics::from_severity(DiagnosticSeverity::Trace);
        assert!(sink.reports(DiagnosticSeverity::Trace));
        assert!(sink.reports(DiagnosticSeverity::Critical));
    }

    #[test]
    fn a_higher_floor_keeps_warnings_out() {
        let sink = StderrDiagnostics::from_severity(DiagnosticSeverity::Error);
        assert!(!sink.reports(DiagnosticSeverity::Warning));
        assert!(sink.reports(DiagnosticSeverity::Error));
    }

    #[test]
    fn the_line_carries_the_code_and_the_message() {
        // The code is what a person greps for and the message is what tells
        // them where; a line missing either is a line that costs a search.
        let line = StderrDiagnostics::format(&warning());
        assert!(line.contains("css.unsupported_function"), "{line}");
        assert!(line.contains("2486..2511"), "{line}");
        assert!(line.contains("warn"), "{line}");
        assert!(line.starts_with("[obscur "), "{line}");
    }

    #[test]
    fn severities_are_ordered_the_way_they_read() {
        assert!(rank(DiagnosticSeverity::Trace) < rank(DiagnosticSeverity::Info));
        assert!(rank(DiagnosticSeverity::Info) < rank(DiagnosticSeverity::Warning));
        assert!(rank(DiagnosticSeverity::Warning) < rank(DiagnosticSeverity::Error));
        assert!(rank(DiagnosticSeverity::Error) < rank(DiagnosticSeverity::Critical));
    }
}
