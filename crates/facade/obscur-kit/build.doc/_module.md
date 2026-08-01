Build-time `.okui` → Rust code generation for the codegen showcase example.

This realises the "`.okui` compiles to ordinary code, not a runtime
interpreter" model: the authored [`showcase.okui`](examples/showcase.okui) is
parsed, compiled and lowered to a runtime `View` **here, at build time**, and
the generated `pub fn showcase_view() -> View` is written into `OUT_DIR`. The
`showcase_okui_codegen_png` example `include!`s it and renders it, so no
`.okui` parsing/compiling machinery is present when that example runs.

The generation always runs (it is cheap — one small document) but only the
example consumes the output, so ordinary library builds are unaffected.
