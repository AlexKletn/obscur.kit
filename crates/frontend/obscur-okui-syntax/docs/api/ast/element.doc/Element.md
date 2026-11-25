Typed AST wrapper over an element-shaped syntax node (spec §2.2).

[`cast`](Self::cast) accepts any of the three element-shaped node kinds —
[`Element`](SyntaxKind::Element), [`Conditional`](SyntaxKind::Conditional)
(`<If>`/`<ElseIf>`/`<Else>`) and [`ForLoop`](SyntaxKind::ForLoop)
(`<For>`/`<VirtualFor>`) — because they all share the `<Tag …>…</Tag>`
shape: an [`OpenTag`](SyntaxKind::OpenTag) node holding the tag name and
attributes, optional content children, and (unless self-closing) a
[`CloseTag`](SyntaxKind::CloseTag) node.

[`is_conditional`](Self::is_conditional) / [`is_loop`](Self::is_loop) recover
the control-flow classification; the dedicated
[`Conditional`](crate::ast::Conditional) / [`LoopExpression`](crate::ast::LoopExpression)
wrappers expose their extra accessors.
