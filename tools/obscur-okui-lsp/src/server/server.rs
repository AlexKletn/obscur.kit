//! `Server`: the JSON-RPC 2.0 dispatch loop over the LSP base protocol.

use std::io::{self, BufRead, Write};

use obscur_okui_compiler::compile::ContractRegistry;

use crate::analysis::{Analyzer, DocumentSymbol, SymbolKind};
use crate::diagnostics::{LspDiagnostic, Position, Range};
use crate::document::LineIndex;
use crate::json::{JsonValue, parse};
use crate::transport::{read_message, write_message};

/// JSON-RPC "method not found" error code.
const METHOD_NOT_FOUND: i64 = -32601;

const TEXT_DOCUMENT_SYNC_FULL: i64 = 1;

/// Converts a `usize` byte offset to `u32`, saturating at [`u32::MAX`].
fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[doc = include_str!("../../docs/api/server/server.doc/Server.md")]
#[derive(Debug, Clone, Default)]
pub struct Server {
    analyzer: Analyzer,
    shutdown_requested: bool,
}

impl Server {
    /// Creates a server with an empty document store and the built-in
    /// primitive/widget contracts registered (whatever [`Analyzer::new`] seeds).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the contract registry the analyzer's compiler consults (fluent),
    /// **replacing** the built-in set [`new`](Self::new) seeded — see
    /// [`Analyzer::with_registry`].
    #[must_use]
    pub fn with_registry(mut self, registry: ContractRegistry) -> Self {
        self.analyzer = Analyzer::new().with_registry(registry);
        self
    }

    #[doc = include_str!("../../docs/api/server/server.doc/run.md")]
    pub fn run<R: BufRead, W: Write>(&mut self, reader: &mut R, writer: &mut W) -> io::Result<()> {
        while let Some(body) = read_message(reader)? {
            let Ok(message) = parse(&body) else {
                // A frame we cannot parse and cannot attribute to an id: skip it.
                continue;
            };
            let method = message
                .get("method")
                .and_then(JsonValue::as_str)
                .unwrap_or("")
                .to_owned();

            if let Some(id) = message.get("id").cloned() {
                let reply = match self.handle_request(&method, &message) {
                    Ok(result) => response(id, result),
                    Err((code, msg)) => error_response(id, code, &msg),
                };
                write_message(writer, &reply.to_string())?;
            } else if method == "exit" {
                return Ok(());
            } else {
                for note in self.handle_notification(&method, &message) {
                    write_message(writer, &note.to_string())?;
                }
            }
        }
        Ok(())
    }

    /// Handles a request (a message carrying an `id`), returning its `result` or
    /// a `(code, message)` JSON-RPC error.
    fn handle_request(
        &mut self,
        method: &str,
        message: &JsonValue,
    ) -> Result<JsonValue, (i64, String)> {
        match method {
            "initialize" => Ok(server_capabilities()),
            "shutdown" => {
                self.shutdown_requested = true;
                Ok(JsonValue::Null)
            }
            "textDocument/documentSymbol" => Ok(self.document_symbols(message)),
            "textDocument/hover" => Ok(self.hover(message)),
            other => Err((METHOD_NOT_FOUND, format!("method not found: {other}"))),
        }
    }

    /// Handles a notification, returning any notifications to send back (e.g.
    /// `textDocument/publishDiagnostics`).
    fn handle_notification(&mut self, method: &str, message: &JsonValue) -> Vec<JsonValue> {
        match method {
            "textDocument/didOpen" => self.did_open(message),
            "textDocument/didChange" => self.did_change(message),
            "textDocument/didClose" => self.did_close(message),
            // `initialized`, `$/setTrace`, etc. need no reply.
            _ => Vec::new(),
        }
    }

    fn did_open(&mut self, message: &JsonValue) -> Vec<JsonValue> {
        let doc = message.get("params").and_then(|p| p.get("textDocument"));
        let Some(uri) = doc.and_then(|d| d.get("uri")).and_then(JsonValue::as_str) else {
            return Vec::new();
        };
        let version = doc
            .and_then(|d| d.get("version"))
            .and_then(JsonValue::as_i64);
        let text = doc
            .and_then(|d| d.get("text"))
            .and_then(JsonValue::as_str)
            .unwrap_or("");
        #[allow(clippy::cast_possible_truncation)]
        self.analyzer
            .store_mut()
            .open(uri, version.unwrap_or(1) as i32, text);
        vec![self.publish_diagnostics(uri)]
    }

    fn did_change(&mut self, message: &JsonValue) -> Vec<JsonValue> {
        let params = message.get("params");
        let doc = params.and_then(|p| p.get("textDocument"));
        let Some(uri) = doc.and_then(|d| d.get("uri")).and_then(JsonValue::as_str) else {
            return Vec::new();
        };
        let uri = uri.to_owned();
        let version = doc
            .and_then(|d| d.get("version"))
            .and_then(JsonValue::as_i64);
        // Full-text sync: the last content change carries the whole document.
        let text = params
            .and_then(|p| p.get("contentChanges"))
            .and_then(JsonValue::as_array)
            .and_then(|changes| changes.last())
            .and_then(|change| change.get("text"))
            .and_then(JsonValue::as_str)
            .unwrap_or("")
            .to_owned();
        #[allow(clippy::cast_possible_truncation)]
        let changed = self
            .analyzer
            .store_mut()
            .change(&uri, version.unwrap_or(1) as i32, text);
        if changed {
            vec![self.publish_diagnostics(&uri)]
        } else {
            Vec::new()
        }
    }

    fn did_close(&mut self, message: &JsonValue) -> Vec<JsonValue> {
        let Some(uri) = message
            .get("params")
            .and_then(|p| p.get("textDocument"))
            .and_then(|d| d.get("uri"))
            .and_then(JsonValue::as_str)
        else {
            return Vec::new();
        };
        let uri = uri.to_owned();
        self.analyzer.store_mut().close(&uri);
        // Clear diagnostics for the now-closed document.
        vec![self.publish_diagnostics(&uri)]
    }

    fn document_symbols(&self, message: &JsonValue) -> JsonValue {
        let Some((uri, text)) = self.uri_and_text(message) else {
            return JsonValue::Array(Vec::new());
        };
        let index = LineIndex::new(&text);
        let symbols = self.analyzer.document_symbols(&uri);
        JsonValue::Array(
            symbols
                .iter()
                .map(|symbol| symbol_to_json(&text, &index, symbol))
                .collect(),
        )
    }

    fn hover(&self, message: &JsonValue) -> JsonValue {
        let Some((uri, text)) = self.uri_and_text(message) else {
            return JsonValue::Null;
        };
        let index = LineIndex::new(&text);
        let position = message.get("params").and_then(|p| p.get("position"));
        let line = position
            .and_then(|p| p.get("line"))
            .and_then(JsonValue::as_u32);
        let character = position
            .and_then(|p| p.get("character"))
            .and_then(JsonValue::as_u32);
        let (Some(line), Some(character)) = (line, character) else {
            return JsonValue::Null;
        };
        let analyzer_pos = lsp_to_analyzer_position(&text, &index, line, character);
        match self.analyzer.hover(&uri, analyzer_pos) {
            Some(hover) => JsonValue::object([
                (
                    "contents",
                    JsonValue::object([
                        ("kind", JsonValue::string("plaintext")),
                        ("value", JsonValue::string(hover.contents.to_string())),
                    ]),
                ),
                ("range", range_to_json(&text, &index, hover.range)),
            ]),
            None => JsonValue::Null,
        }
    }

    /// Builds a `textDocument/publishDiagnostics` notification for `uri` from the
    /// current analyzer state (empty diagnostics if the document is closed).
    fn publish_diagnostics(&self, uri: &str) -> JsonValue {
        let document = self.analyzer.store().get(uri);
        let text = document.map(|d| d.text.clone()).unwrap_or_default();
        let version = document.map(|d| i64::from(d.version));
        let index = LineIndex::new(&text);
        let diagnostics = self.analyzer.analyze(uri);
        let items: Vec<JsonValue> = diagnostics
            .iter()
            .map(|diagnostic| diagnostic_to_json(&text, &index, diagnostic))
            .collect();
        let mut params = vec![
            ("uri".to_owned(), JsonValue::string(uri)),
            ("diagnostics".to_owned(), JsonValue::Array(items)),
        ];
        if let Some(version) = version {
            params.push(("version".to_owned(), JsonValue::int(version)));
        }
        notification("textDocument/publishDiagnostics", JsonValue::Object(params))
    }

    /// Extracts `params.textDocument.uri` and clones the document's current text.
    fn uri_and_text(&self, message: &JsonValue) -> Option<(String, String)> {
        let uri = message
            .get("params")
            .and_then(|p| p.get("textDocument"))
            .and_then(|d| d.get("uri"))
            .and_then(JsonValue::as_str)?;
        let text = self
            .analyzer
            .store()
            .get(uri)
            .map(|d| d.text.clone())
            .unwrap_or_default();
        Some((uri.to_owned(), text))
    }
}

/// The `initialize` result: advertised server capabilities.
fn server_capabilities() -> JsonValue {
    JsonValue::object([
        (
            "capabilities",
            JsonValue::object([
                ("textDocumentSync", JsonValue::int(TEXT_DOCUMENT_SYNC_FULL)),
                ("documentSymbolProvider", JsonValue::Bool(true)),
                ("hoverProvider", JsonValue::Bool(true)),
            ]),
        ),
        (
            "serverInfo",
            JsonValue::object([
                ("name", JsonValue::string("obscur-okui-lsp")),
                ("version", JsonValue::string(env!("CARGO_PKG_VERSION"))),
            ]),
        ),
    ])
}

/// Builds a JSON-RPC success response echoing `id`.
fn response(id: JsonValue, result: JsonValue) -> JsonValue {
    JsonValue::object([
        ("jsonrpc", JsonValue::string("2.0")),
        ("id", id),
        ("result", result),
    ])
}

/// Builds a JSON-RPC error response echoing `id`.
fn error_response(id: JsonValue, code: i64, message: &str) -> JsonValue {
    JsonValue::object([
        ("jsonrpc", JsonValue::string("2.0")),
        ("id", id),
        (
            "error",
            JsonValue::object([
                ("code", JsonValue::int(code)),
                ("message", JsonValue::string(message)),
            ]),
        ),
    ])
}

/// Builds a JSON-RPC notification.
fn notification(method: &str, params: JsonValue) -> JsonValue {
    JsonValue::object([
        ("jsonrpc", JsonValue::string("2.0")),
        ("method", JsonValue::string(method)),
        ("params", params),
    ])
}

/// Serializes an [`LspDiagnostic`] into an LSP `Diagnostic` object.
fn diagnostic_to_json(text: &str, index: &LineIndex, diagnostic: &LspDiagnostic) -> JsonValue {
    JsonValue::object([
        ("range", range_to_json(text, index, diagnostic.range)),
        (
            "severity",
            JsonValue::int(i64::from(diagnostic.severity.code())),
        ),
        ("source", JsonValue::string(diagnostic.source)),
        ("message", JsonValue::string(diagnostic.message.to_string())),
    ])
}

/// Serializes a [`DocumentSymbol`] into an LSP `DocumentSymbol` object.
fn symbol_to_json(text: &str, index: &LineIndex, symbol: &DocumentSymbol) -> JsonValue {
    let range = range_to_json(text, index, symbol.range);
    JsonValue::object([
        ("name", JsonValue::string(symbol.name.to_string())),
        ("detail", JsonValue::string(symbol.kind.label())),
        (
            "kind",
            JsonValue::int(i64::from(lsp_symbol_kind(symbol.kind))),
        ),
        ("range", range.clone()),
        ("selectionRange", range),
        (
            "children",
            JsonValue::Array(
                symbol
                    .children
                    .iter()
                    .map(|child| symbol_to_json(text, index, child))
                    .collect(),
            ),
        ),
    ])
}

/// Maps an okui [`SymbolKind`] to an LSP `SymbolKind` numeric code
/// (`Class = 5`, `Field = 8`).
fn lsp_symbol_kind(kind: SymbolKind) -> u8 {
    match kind {
        SymbolKind::Component => 5,
        SymbolKind::Element => 8,
    }
}

/// Serializes an analyzer [`Range`] into an LSP `Range`, re-encoding byte columns
/// to UTF-16 code units.
fn range_to_json(text: &str, index: &LineIndex, range: Range) -> JsonValue {
    JsonValue::object([
        ("start", position_to_json(text, index, range.start)),
        ("end", position_to_json(text, index, range.end)),
    ])
}

/// Serializes an analyzer [`Position`] (byte column) into an LSP `Position`
/// (UTF-16 code-unit column).
fn position_to_json(text: &str, index: &LineIndex, position: Position) -> JsonValue {
    let (line, character) = analyzer_to_lsp_position(text, index, position);
    JsonValue::object([
        ("line", JsonValue::int(i64::from(line))),
        ("character", JsonValue::int(i64::from(character))),
    ])
}

/// Converts an analyzer byte-column [`Position`] to an LSP `(line, utf16_char)`.
fn analyzer_to_lsp_position(text: &str, index: &LineIndex, position: Position) -> (u32, u32) {
    let line_start = index.position_to_offset(Position::new(position.line, 0));
    let target = line_start + position.character as usize;
    let mut byte = line_start;
    let mut utf16 = 0u32;
    for ch in text.get(line_start..).unwrap_or("").chars() {
        if byte >= target {
            break;
        }
        byte += ch.len_utf8();
        utf16 += to_u32(ch.len_utf16());
    }
    (position.line, utf16)
}

/// Converts an LSP `(line, utf16_char)` position to an analyzer byte-column
/// [`Position`] via the document's [`LineIndex`].
fn lsp_to_analyzer_position(text: &str, index: &LineIndex, line: u32, character: u32) -> Position {
    let line_start = index.position_to_offset(Position::new(line, 0));
    let mut byte = line_start;
    let mut utf16 = 0u32;
    for ch in text.get(line_start..).unwrap_or("").chars() {
        if ch == '\n' || utf16 >= character {
            break;
        }
        utf16 += to_u32(ch.len_utf16());
        byte += ch.len_utf8();
    }
    index.offset_to_position(to_u32(byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use obscur_okui_compiler::validation::ComponentContract;
    use std::io::Cursor;

    /// Frames a batch of JSON bodies as an LSP input stream.
    fn frame_all(bodies: &[&str]) -> Vec<u8> {
        let mut out = Vec::new();
        for body in bodies {
            write_message(&mut out, body).unwrap();
        }
        out
    }

    /// Splits a written LSP stream back into parsed JSON messages.
    fn parse_all(bytes: Vec<u8>) -> Vec<JsonValue> {
        let mut cursor = Cursor::new(bytes);
        let mut out = Vec::new();
        while let Some(body) = read_message(&mut cursor).unwrap() {
            out.push(parse(&body).unwrap());
        }
        out
    }

    fn run_server(bodies: &[&str]) -> Vec<JsonValue> {
        let registry = ContractRegistry::new().with(
            ComponentContract::new("Button")
                .requiring_prop("label")
                .allowing_event("click"),
        );
        let mut server = Server::new().with_registry(registry);
        let mut input = Cursor::new(frame_all(bodies));
        let mut output = Vec::new();
        server.run(&mut input, &mut output).unwrap();
        parse_all(output)
    }

    #[test]
    fn initialize_returns_capabilities() {
        let out =
            run_server(&["{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{}}"]);
        assert_eq!(out.len(), 1);
        let caps = out[0]
            .get("result")
            .and_then(|r| r.get("capabilities"))
            .unwrap();
        assert_eq!(
            caps.get("textDocumentSync").and_then(JsonValue::as_i64),
            Some(1)
        );
        assert_eq!(caps.get("hoverProvider"), Some(&JsonValue::Bool(true)));
        assert_eq!(
            caps.get("documentSymbolProvider"),
            Some(&JsonValue::Bool(true))
        );
        // The id round-trips exactly.
        assert_eq!(out[0].get("id").and_then(JsonValue::as_i64), Some(1));
    }

    #[test]
    fn did_open_broken_document_publishes_diagnostics() {
        // A missing close tag is a recoverable parse error.
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\",\"version\":1,\"text\":\"<Button>oops\"}}}",
        ]);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].get("method").and_then(JsonValue::as_str),
            Some("textDocument/publishDiagnostics")
        );
        let params = out[0].get("params").unwrap();
        assert_eq!(
            params.get("uri").and_then(JsonValue::as_str),
            Some("file:///a.okui")
        );
        let diags = params
            .get("diagnostics")
            .and_then(JsonValue::as_array)
            .unwrap();
        assert!(!diags.is_empty(), "expected at least one diagnostic");
        // Diagnostics carry a range and a severity code.
        let first = &diags[0];
        assert!(first.get("range").and_then(|r| r.get("start")).is_some());
        assert_eq!(first.get("severity").and_then(JsonValue::as_i64), Some(1));
    }

    #[test]
    fn did_change_republishes_diagnostics() {
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\",\"version\":1,\"text\":\"<Button label=\\\"Save\\\"/>\"}}}",
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didChange\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\",\"version\":2},\"contentChanges\":[{\"text\":\"<Button>oops\"}]}}",
        ]);
        assert_eq!(out.len(), 2);
        // First publish (well-formed) has no diagnostics; second (broken) has some.
        let first = out[0]
            .get("params")
            .and_then(|p| p.get("diagnostics"))
            .and_then(JsonValue::as_array)
            .unwrap();
        assert!(first.is_empty(), "well-formed document: {first:?}");
        let second = out[1]
            .get("params")
            .and_then(|p| p.get("diagnostics"))
            .and_then(JsonValue::as_array)
            .unwrap();
        assert!(!second.is_empty());
        assert_eq!(
            out[1]
                .get("params")
                .and_then(|p| p.get("version"))
                .and_then(JsonValue::as_i64),
            Some(2)
        );
    }

    #[test]
    fn document_symbol_returns_outline() {
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\",\"version\":1,\"text\":\"<Col><Row/></Col>\"}}}",
            "{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"textDocument/documentSymbol\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\"}}}",
        ]);
        // out[0] is publishDiagnostics; out[1] is the documentSymbol response.
        let result = out[1].get("result").and_then(JsonValue::as_array).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].get("name").and_then(JsonValue::as_str),
            Some("Col")
        );
        assert_eq!(result[0].get("kind").and_then(JsonValue::as_i64), Some(5));
        let children = result[0]
            .get("children")
            .and_then(JsonValue::as_array)
            .unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(
            children[0].get("name").and_then(JsonValue::as_str),
            Some("Row")
        );
    }

    #[test]
    fn hover_returns_contents_for_the_element_under_the_cursor() {
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\",\"version\":1,\"text\":\"<Col><Row/></Col>\"}}}",
            // Byte 6 is inside `<Row/>` (which spans 5..11).
            "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"textDocument/hover\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\"},\"position\":{\"line\":0,\"character\":6}}}",
        ]);
        let result = out[1].get("result").unwrap();
        let value = result
            .get("contents")
            .and_then(|c| c.get("value"))
            .and_then(JsonValue::as_str)
            .unwrap();
        assert!(value.contains("Row"), "hover contents: {value}");
        assert!(result.get("range").is_some());
    }

    #[test]
    fn hover_outside_any_element_is_null() {
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\",\"version\":1,\"text\":\"  <A/>\"}}}",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"textDocument/hover\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\"},\"position\":{\"line\":0,\"character\":0}}}",
        ]);
        assert_eq!(out[1].get("result"), Some(&JsonValue::Null));
    }

    #[test]
    fn unknown_request_yields_method_not_found() {
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"textDocument/completion\",\"params\":{}}",
        ]);
        assert_eq!(
            out[0]
                .get("error")
                .and_then(|e| e.get("code"))
                .and_then(JsonValue::as_i64),
            Some(METHOD_NOT_FOUND)
        );
    }

    #[test]
    fn shutdown_then_exit_stops_cleanly() {
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"shutdown\",\"params\":null}",
            "{\"jsonrpc\":\"2.0\",\"method\":\"exit\"}",
            // Anything after exit must not be processed.
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"initialize\",\"params\":{}}",
        ]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].get("id").and_then(JsonValue::as_i64), Some(1));
        assert_eq!(out[0].get("result"), Some(&JsonValue::Null));
    }

    #[test]
    fn did_close_clears_diagnostics() {
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\",\"version\":1,\"text\":\"<Button>oops\"}}}",
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didClose\",\"params\":{\"textDocument\":{\"uri\":\"file:///a.okui\"}}}",
        ]);
        assert_eq!(out.len(), 2);
        let cleared = out[1]
            .get("params")
            .and_then(|p| p.get("diagnostics"))
            .and_then(JsonValue::as_array)
            .unwrap();
        assert!(cleared.is_empty());
    }

    #[test]
    fn utf16_columns_are_reencoded_for_non_ascii_source() {
        // 'é' is one UTF-16 unit but two UTF-8 bytes; `<café/>` — the tag spans
        // bytes 0..8 but UTF-16 columns 0..7.
        let out = run_server(&[
            "{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{\"textDocument\":{\"uri\":\"file:///u.okui\",\"version\":1,\"text\":\"<café/>\"}}}",
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"textDocument/documentSymbol\",\"params\":{\"textDocument\":{\"uri\":\"file:///u.okui\"}}}",
        ]);
        let result = out[1].get("result").and_then(JsonValue::as_array).unwrap();
        let end = result[0].get("range").and_then(|r| r.get("end")).unwrap();
        assert_eq!(end.get("character").and_then(JsonValue::as_i64), Some(7));
    }
}
