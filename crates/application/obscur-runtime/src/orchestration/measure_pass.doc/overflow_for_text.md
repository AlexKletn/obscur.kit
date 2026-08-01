Resolves the used `text-overflow` for an anonymous text `node` (CSS UI 3
§5.2).

`text-overflow` applies to the block container and does **not** inherit, so
the anonymous text node's own computed value is always the initial `clip`.
The value that governs the node's inline content is the block container's, so
it is read from the text node's parent, falling back to the node itself when
there is none. (Nested inline boxes are not walked to the container — a
documented approximation matching this coordinator's flat text model.)
