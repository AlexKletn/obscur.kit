The analysis core of the `.okui` language server.

The `Analyzer` owns a [`DocumentStore`] (mutate it via [`store_mut`](Self::store_mut)
to drive the `didOpen`/`didChange`/`didClose` lifecycle) and a
[`ContractRegistry`] used by the compiler's §2.13 contract checks. It is
transport-agnostic: a JSON-RPC/stdio wrapper feeds documents into the store
and calls [`analyze`](Self::analyze), [`document_symbols`](Self::document_symbols)
and [`hover`](Self::hover), then serializes the results onto the wire.

# Diagnostics

[`analyze`](Self::analyze) parses the document with the okui
[`Parser`], mapping each parse error (a byte offset) to a one-character
[`LspDiagnostic`] via a [`LineIndex`]. When the source parses cleanly it is
cast to a [`Document`] and run through the okui [`Compiler`]; every
[`CompileDiagnostic`] is likewise mapped from its byte offset to an
[`LspDiagnostic`].

# Why the default registry is [`ContractRegistry::builtins`]

It used to be `ContractRegistry::default()` — **empty** — and that made this
server, the one tool an author actually points at a `.okui` file, enforce
§2.13 for *nothing*. `Compiler::new` seeds the built-in primitive and widget
contracts precisely so a default compile rejects `<Button laebl="Save"/>`;
[`analyze`](Self::analyze) then called `.with_registry(…)`, which **replaces**
that seed wholesale, and handed it the empty default straight back. So
`obscur-okui-lsp --check` reported `diagnostics: none` and exited zero on a
document the very same `Compiler` rejects in `obscur-kit`'s `build.rs` and the
`okui!` macro rejects at the author's own token.

Seeding the same registry `Compiler::new` does makes the editor and the build
agree by construction. [`with_registry`](Self::with_registry) still replaces
it wholesale — a host wiring its own components in wants
`ContractRegistry::builtins().with_surface(&MyProps::contract_surface())`, not
`ContractRegistry::new()`, or it will silently drop the built-ins exactly as
this did.
