//! Diagnostics: the transport-agnostic [`LspDiagnostic`] plus its coordinate
//! ([`Position`], [`Range`]) and [`LspSeverity`] vocabulary.
mod lsp_diagnostic;
pub use lsp_diagnostic::LspDiagnostic;
mod position;
pub use position::Position;
mod range;
pub use range::Range;
mod severity;
pub use severity::LspSeverity;
