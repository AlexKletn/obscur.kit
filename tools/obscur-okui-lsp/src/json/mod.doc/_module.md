A minimal, hand-rolled JSON value, parser and serializer.

This module deliberately avoids `serde`/`serde_json`: like the rest of the
workspace (the CSS tokenizer, the `.okui` parser, the LSP analyzer core) the
JSON support is hand-written and `std`-only, scoped to exactly the shapes the
LSP wire protocol needs — objects, arrays, strings, numbers, booleans and
null, with correct string escaping and UTF-8 handling.
