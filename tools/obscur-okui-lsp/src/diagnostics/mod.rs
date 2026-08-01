#![doc = include_str!("mod.doc/_module.md")]
mod lsp_diagnostic;
pub use lsp_diagnostic::LspDiagnostic;
mod position;
pub use position::{Position, Range};
mod severity;
pub use severity::LspSeverity;
