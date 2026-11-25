The button's semantic node, ready to place and style — [`view`](Self::view)
reduced to its single root [`ViewNode`].

A button always renders exactly one node, so this is infallible where the
general [`View::into_single_node`] is not: it asserts that invariant here,
once, and hands callers a bare node instead of making each of them unwrap a
`View`. Use it wherever the surrounding tree wants a node rather than a
`View` — e.g. `Button::new(label, cmd).into_view_node().class("bar")`.

# Panics

Never in practice: [`view`](Self::view) is a `View::single`, so it always
has exactly one root node. The internal `expect` guards that invariant and
would only fire if `view` were changed to emit zero or many roots.
