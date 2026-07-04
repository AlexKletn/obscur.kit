//! The analysis core of the `.okui` language server.
//!
//! This crate is a **transport-agnostic** language-server core: it depends only
//! on `std` and the two okui frontend crates (`obscur-okui-syntax`,
//! `obscur-okui-compiler`) — no JSON-RPC, serde or LSP framework. It provides
//! the pieces a real LSP server is built from:
//!
//! - [`DocumentStore`] / [`TextDocument`] — the open-document set with full-text
//!   synchronization (the `didOpen`/`didChange`/`didClose` lifecycle).
//! - [`LineIndex`] — byte-offset ↔ [`Position`] mapping (UTF-8 byte columns).
//! - [`Analyzer`] — parses and compiles a document and produces
//!   [`LspDiagnostic`]s, [`DocumentSymbol`] outlines and [`Hover`] results.
//!
//! A JSON-RPC/stdio wrapper (see the crate's binary) is a thin adapter: it
//! translates `textDocument/didOpen` → [`DocumentStore::open`],
//! `textDocument/didChange` → [`DocumentStore::change`],
//! `textDocument/didClose` → [`DocumentStore::close`], then runs
//! [`Analyzer::analyze`] and publishes the result via
//! `textDocument/publishDiagnostics`. `textDocument/documentSymbol` and
//! `textDocument/hover` map to [`Analyzer::document_symbols`] and
//! [`Analyzer::hover`].
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod analysis;
pub mod diagnostics;
pub mod document;

pub use analysis::{Analyzer, DocumentSymbol, Hover, SymbolKind};
pub use diagnostics::{LspDiagnostic, LspSeverity, Position, Range};
pub use document::{DocumentStore, LineIndex, TextDocument};
