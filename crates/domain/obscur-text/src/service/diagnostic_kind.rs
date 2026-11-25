/// The category of a [`TextDiagnostic`](super::TextDiagnostic) — which language service produced it
/// (spec §5.13 optional language services).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticKind {
    /// A misspelled word flagged by spellcheck.
    Spelling,
    /// A grammatical issue flagged by the grammar checker.
    Grammar,
    /// An autocorrect suggestion (a likely typo with a replacement).
    Autocorrect,
    /// A stylistic remark (wordiness, tone, etc.).
    Style,
}
