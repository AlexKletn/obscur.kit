The TSX-like proc-macro DSL and the `#[derive(Contract)]` derive (spec §2.8,
§2.13).

# `okui!` compiles to ordinary code, not a runtime

[`okui!`] is the spec's *primary* authoring surface (the `.okui` file codegen
is the fallback). It does **not** hand a document to an interpreter: during
expansion the DSL is parsed, lowered to UI IR, and run through `ViewCodegen`
— the same lowering the `.okui` build step performs — so what lands in the
crate is plain Rust building a `View`. No IR, no `.okui` machinery and no
interpretation exist at runtime.

Because this is a `proc-macro` crate it can export only macros; the expansion
logic lives in the private `dsl` / `derive` modules and is covered by their
own unit tests.
