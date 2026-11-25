A hierarchical outline entry for one element/component in a `.okui` document.

Produced by [`Analyzer::document_symbols`](crate::Analyzer::document_symbols),
it carries the tag `name`, its [`SymbolKind`], the source `range` it covers,
and its nested `children`, so a client can render a `textDocument/documentSymbol`
outline.
