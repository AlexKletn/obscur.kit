#![doc = include_str!("diagnostic_severity.doc/_module.md")]

#[doc = include_str!("diagnostic_severity.doc/DiagnosticSeverity.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticSeverity {
    #[doc = include_str!("diagnostic_severity.doc/Trace.md")]
    Trace,
    #[doc = include_str!("diagnostic_severity.doc/Info.md")]
    Info,
    #[doc = include_str!("diagnostic_severity.doc/Warning.md")]
    Warning,
    #[doc = include_str!("diagnostic_severity.doc/Error.md")]
    Error,
    #[doc = include_str!("diagnostic_severity.doc/Critical.md")]
    Critical,
}
