//! `.okui` language-server binary.
//!
//! By default this speaks the real Language Server Protocol over stdio: a
//! JSON-RPC 2.0 loop (LSP base protocol, `Content-Length`-framed messages) built
//! on the transport-agnostic core in the [`obscur_okui_lsp`] library. It handles
//! `initialize`/`shutdown`/`exit`, the `textDocument/didOpen`/`didChange`/
//! `didClose` lifecycle (pushing `textDocument/publishDiagnostics`),
//! `textDocument/documentSymbol` and `textDocument/hover`, delegating all
//! analysis to the [`Analyzer`](obscur_okui_lsp::Analyzer)/[`Server`].
//!
//! With `--check` it instead runs a single-shot `.okui` checker: it reads source
//! from standard input, prints the diagnostics (`line:col severity [source]
//! message`) and the document-symbol outline, and exits non-zero when any
//! error-severity diagnostic is found, so it is usable in a CI check.
#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::process::ExitCode;

use obscur_okui_lsp::{Analyzer, DocumentSymbol, LspDiagnostic, LspSeverity, Server};

/// The synthetic URI used for the document read from standard input in `--check`.
const STDIN_URI: &str = "stdin://input.okui";

fn main() -> ExitCode {
    if std::env::args().any(|arg| arg == "--check") {
        return run_check();
    }
    run_server()
}

/// Runs the JSON-RPC/stdio language server over real standard input/output.
fn run_server() -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = stdin.lock();
    let mut writer = stdout.lock();
    match Server::new().run(&mut reader, &mut writer) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("obscur-okui-lsp: transport error: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Runs the single-shot `.okui` checker over standard input (the `--check` mode).
fn run_check() -> ExitCode {
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

    let has_errors = diagnostics.iter().any(|d| d.severity == LspSeverity::Error);
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
