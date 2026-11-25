Compiles a [`ParsedStylesheet`] into a [`CompiledStylesheet`] (§4.12).

The compiler owns the string interner, the selector-id counter and the
document-order counter used to rank declarations (§4.14). Construct one, set
the [`CascadeOrigin`] with [`with_origin`](StylesheetCompiler::with_origin)
if the default (`GlobalApp`) is not right, then
[`compile`](StylesheetCompiler::compile).
