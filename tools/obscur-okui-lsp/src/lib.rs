#![doc = include_str!("../docs/api/lib.doc/_module.md")]
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod analysis;
pub mod diagnostics;
pub mod document;
pub mod json;
pub mod server;
pub mod transport;

pub use analysis::{Analyzer, DocumentSymbol, Hover, SymbolKind};
pub use diagnostics::{LspDiagnostic, LspSeverity, Position, Range};
pub use document::{DocumentStore, LineIndex, TextDocument};
pub use server::Server;
