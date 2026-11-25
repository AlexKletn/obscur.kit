A single diagnostic in LSP shape: a source [`Range`], a [`LspSeverity`], a
human-readable `message` and a `source` label naming the producing pass.

This is the core's uniform diagnostic currency: parse errors from the okui
parser and semantic diagnostics from the okui compiler are both mapped into
this type by the [`Analyzer`](crate::Analyzer). A JSON-RPC wrapper serializes
it directly into an LSP `Diagnostic` for `textDocument/publishDiagnostics`.
