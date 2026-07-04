//! Analysis: the [`Analyzer`] core plus its result types — [`DocumentSymbol`]
//! outlines and [`Hover`] information.
mod analyzer;
pub use analyzer::Analyzer;
mod document_symbol;
pub use document_symbol::{DocumentSymbol, SymbolKind};
mod hover;
pub use hover::Hover;
