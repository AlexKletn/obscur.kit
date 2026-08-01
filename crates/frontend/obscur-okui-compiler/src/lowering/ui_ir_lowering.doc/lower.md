Lowers `doc` into a complete [`UiIrDocument`], consuming the pass and
appending any non-fatal lowering diagnostics (e.g. a `<For>` missing its
mandatory key, spec §2.11) to `diagnostics`.

# Errors

Returns [`LoweringError::EmptyDocument`] when the document has no
top-level elements, or [`LoweringError::MissingTagName`] when an element
carries no tag name. These are fatal — no document is produced; a
[`LoweringError::MissingLoopKey`] is *not* fatal and is reported through
`diagnostics` while the document is still lowered.
