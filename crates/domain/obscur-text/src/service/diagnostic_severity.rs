#[doc = include_str!("../../docs/api/service/text_diagnostic.doc/DiagnosticSeverity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    /// An informational note; no action implied.
    Info,
    /// A warning the user may wish to address.
    Warning,
    /// An error the user is likely expected to fix.
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_orders_least_to_most_severe() {
        assert!(DiagnosticSeverity::Info < DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Error);
    }
}
