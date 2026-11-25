A JSON-RPC/stdio language server for `.okui`.

The server owns an [`Analyzer`] (which owns the open-document
[`DocumentStore`](crate::DocumentStore)) and drives it from LSP messages:
`textDocument/didOpen`/`didChange`/`didClose` mutate the store and publish
diagnostics; `textDocument/documentSymbol` and `textDocument/hover` are
answered from the analyzer. All JSON is hand-rolled (see [`crate::json`]) —
no `serde`.

# Position units

The analyzer works in UTF-8 byte columns; LSP clients speak UTF-16 code-unit
columns. Incoming positions are converted to byte columns and outgoing ranges
back to UTF-16 against the document's current text, so coordinates are exact
even for non-ASCII source.
