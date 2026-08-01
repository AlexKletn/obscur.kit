Expander for the TSX-like Rust DSL (spec §2.8).

[`DslMacro::to_document`] parses the DSL into a real [`UiIrDocument`] during
expansion; `okui!` then lowers that document through `ViewCodegen`, so the
emitted code builds a `View` directly and no IR exists at runtime.
