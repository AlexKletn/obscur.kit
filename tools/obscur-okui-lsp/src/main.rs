//! `.okui` language-server CLI.
//!
//! This binary is a thin, honest wrapper over the language-server core in the
//! [`obscur_okui_lsp`] library. Pulling a full JSON-RPC framework is out of
//! scope, so instead of speaking the LSP wire protocol this binary runs a
//! single-shot **`.okui` checker**: it reads source from standard input, runs
//! the [`Analyzer`](obscur_okui_lsp::Analyzer), and prints the diagnostics
//! (`line:col severity [source] message`) and the document-symbol outline to
//! standard output. It exits non-zero when any error-severity diagnostic is
//! found, so it is usable in a CI check.
//!
//! A real JSON-RPC/stdio LSP transport is a thin adapter over the same core:
//! `initialize` advertises the capabilities, `textDocument/didOpen` calls
//! [`DocumentStore::open`](obscur_okui_lsp::DocumentStore::open),
//! `textDocument/didChange` calls
//! [`DocumentStore::change`](obscur_okui_lsp::DocumentStore::change),
//! `textDocument/didClose` calls
//! [`DocumentStore::close`](obscur_okui_lsp::DocumentStore::close), and after
//! each edit the server calls [`Analyzer::analyze`](obscur_okui_lsp::Analyzer::analyze)
//! and emits `textDocument/publishDiagnostics`. `textDocument/documentSymbol`
//! and `textDocument/hover` map to
//! [`Analyzer::document_symbols`](obscur_okui_lsp::Analyzer::document_symbols)
//! and [`Analyzer::hover`](obscur_okui_lsp::Analyzer::hover).
#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::process::ExitCode;

use obscur_okui_lsp::{Analyzer, DocumentSymbol, LspDiagnostic, LspSeverity};

/// The synthetic URI used for the document read from standard input.
const STDIN_URI: &str = "stdin://input.okui";

fn main() -> ExitCode {
    let mut source = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut source) {
        eprintln!("obscur-okui-lsp: failed to read stdin: {error}");
        return ExitCode::FAILURE;
    }

    let mut analyzer = Analyzer::new();
    analyzer.store_mut().open(STDIN_URI, 1, source);

    let diagnostics = analyzer.analyze(STDIN_URI);
    let symbols = analyzer.document_symbols(STDIN_URI);

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let has_errors = report(&mut out, &diagnostics, &symbols).unwrap_or(false);

    if has_errors {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Writes the diagnostics and symbol outline to `out`, returning whether any
/// error-severity diagnostic was present.
fn report(
    out: &mut impl Write,
    diagnostics: &[LspDiagnostic],
    symbols: &[DocumentSymbol],
) -> io::Result<bool> {
    if diagnostics.is_empty() {
        writeln!(out, "diagnostics: none")?;
    } else {
        writeln!(out, "diagnostics ({}):", diagnostics.len())?;
        for diagnostic in diagnostics {
            // Report 1-based line/column, which is what editors display.
            writeln!(
                out,
                "  {}:{} {} [{}] {}",
                diagnostic.range.start.line + 1,
                diagnostic.range.start.character + 1,
                diagnostic.severity.label(),
                diagnostic.source,
                diagnostic.message,
            )?;
        }
    }

    writeln!(out, "symbols ({}):", symbols.len())?;
    for symbol in symbols {
        write_symbol(out, symbol, 1)?;
    }

    let has_errors = diagnostics
        .iter()
        .any(|d| d.severity == LspSeverity::Error);
    Ok(has_errors)
}

/// Writes one symbol and its children, indented by `depth` levels.
fn write_symbol(out: &mut impl Write, symbol: &DocumentSymbol, depth: usize) -> io::Result<()> {
    writeln!(
        out,
        "{}{} {} ({}:{}..{}:{})",
        "  ".repeat(depth),
        symbol.kind.label(),
        symbol.name,
        symbol.range.start.line + 1,
        symbol.range.start.character + 1,
        symbol.range.end.line + 1,
        symbol.range.end.character + 1,
    )?;
    for child in &symbol.children {
        write_symbol(out, child, depth + 1)?;
    }
    Ok(())
}
