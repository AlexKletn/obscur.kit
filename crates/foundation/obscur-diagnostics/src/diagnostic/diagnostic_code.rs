#![doc = include_str!("diagnostic_code.doc/_module.md")]

#[doc = include_str!("diagnostic_code.doc/DiagnosticCode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    #[doc = include_str!("diagnostic_code.doc/new.md")]
    #[must_use]
    pub const fn new(code: &'static str) -> Self {
        Self(code)
    }

    #[doc = include_str!("diagnostic_code.doc/as_str.md")]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}
