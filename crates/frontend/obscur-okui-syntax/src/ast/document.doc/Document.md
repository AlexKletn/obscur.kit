Typed AST wrapper over a [`Document`](SyntaxKind::Document) syntax node
(spec §2.2).

A `Document` is the root of a parsed `.okui` source. It wraps the underlying
[`SyntaxNode`] by value (a cheap clone) so the typed view is owned and can be
passed around freely while the lossless tree stays intact underneath.

Its top-level children are elements (including the element-shaped
[`Conditional`](SyntaxKind::Conditional) / [`ForLoop`](SyntaxKind::ForLoop)
control-flow constructs), interpolated expressions and raw text. Only the
element-shaped children are surfaced by [`elements`](Self::elements).
