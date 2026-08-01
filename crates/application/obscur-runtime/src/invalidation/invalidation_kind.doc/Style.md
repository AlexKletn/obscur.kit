A node's computed style changed; requires restyle. Because layout style
is derived from computed style, in Phase 1 this also requires relayout
(and hence repaint + recompose).
