A zero-based position in a text document.

`line` is the zero-based line number (newlines split lines) and `character`
is the zero-based **UTF-8 byte** offset from the start of that line.

# Column unit

The Language Server Protocol defines `character` as a UTF-16 code-unit
offset. This transport-agnostic core instead uses a **UTF-8 byte** column so
coordinates map losslessly to and from the byte offsets carried by the okui
parser and compiler diagnostics (which are byte offsets). A JSON-RPC wrapper
that must speak strict LSP can re-encode the column against the line's text;
for a byte-oriented `.okui` checker the byte column is exact and stable.
