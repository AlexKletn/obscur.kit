Returns `field`'s first text leaf (depth-first, in child order) — the node
the session text is painted on, whose box origin the click-caret x is
measured from. Mirrors the compose walk's editing-text selection: the field
itself if it directly carries text, else its first `Text` descendant. `None`
when the subtree has no text node.
