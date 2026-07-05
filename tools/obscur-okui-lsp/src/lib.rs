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
//! On top of that core the crate provides a real transport:
//!
//! - [`json`] — a hand-rolled (`serde`-free) JSON value, parser and serializer.
//! - [`transport`] — `Content-Length`-framed LSP base-protocol message I/O.
//! - [`Server`] — the JSON-RPC 2.0/stdio dispatch loop that wires the LSP
//!   `initialize`/`shutdown`, `textDocument/didOpen`/`didChange`/`didClose`
//!   (→ `publishDiagnostics`), `textDocument/documentSymbol` and
//!   `textDocument/hover` messages onto the [`Analyzer`].
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
