`display: contents`: the node generates no box of its own; its children
are hoisted into the parent's formatting context in its place (css-display-3
§2.5). Unlike [`None`], the subtree still participates in layout — only the
element's own box is absent.
