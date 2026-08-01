End-to-end proof that the spec's §2.8 **primary** authoring surface — the
`okui!` proc-macro — exists, expands, and compiles to ordinary code.

The macro is the point at which "`.okui` compiles to ordinary Rust, not a
runtime" has to hold for the *inline* path: `okui!` parses the TSX-like DSL,
lowers it to UI IR **while the macro runs**, and emits the `View`-building
Rust that `ViewCodegen` produces. Nothing here interprets an IR document at
runtime — the assertions below inspect a plain `View`.

These tests are the call site the audit found missing: previously
`DslMacro::expand` was solid logic that no macro invoked.
