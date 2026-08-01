Materializes a balanced [`ParseEvent`] stream into a [`SyntaxNode`] tree and
its collected [`ParseError`]s.

An explicit stack of in-progress nodes is maintained:
[`StartNode`](ParseEvent::StartNode) pushes a node,
[`Token`](ParseEvent::Token) attaches to the top node,
[`FinishNode`](ParseEvent::FinishNode) pops the top and attaches it to the
new top (or, for the outermost node, becomes the root), and
[`Error`](ParseEvent::Error) is collected. Any unbalanced remainder is folded
defensively so a root is always produced.
