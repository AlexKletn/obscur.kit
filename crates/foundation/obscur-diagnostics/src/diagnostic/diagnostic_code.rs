//! Stable diagnostic code.

/// A stable, machine-readable diagnostic code. Codes are part of the
/// diagnostics contract and must not be reused for unrelated conditions
/// (spec §1.6, §7.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    /// Creates a code from a stable string literal.
    #[must_use]
    pub const fn new(code: &'static str) -> Self {
        Self(code)
    }

    /// Returns the code string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}
