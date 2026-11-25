`.okui` language-server binary.

By default this speaks the real Language Server Protocol over stdio: a
JSON-RPC 2.0 loop (LSP base protocol, `Content-Length`-framed messages) built
on the transport-agnostic core in the [`obscur_okui_lsp`] library. It handles
`initialize`/`shutdown`/`exit`, the `textDocument/didOpen`/`didChange`/
`didClose` lifecycle (pushing `textDocument/publishDiagnostics`),
`textDocument/documentSymbol` and `textDocument/hover`, delegating all
analysis to the [`Analyzer`](obscur_okui_lsp::Analyzer)/[`Server`].

With `--check` it instead runs a single-shot `.okui` checker: it reads source
from standard input, prints the diagnostics (`line:col severity [source]
message`) and the document-symbol outline, and exits non-zero when any
error-severity diagnostic is found, so it is usable in a CI check.
