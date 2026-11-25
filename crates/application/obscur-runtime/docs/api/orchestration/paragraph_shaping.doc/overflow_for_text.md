Resolves the used `text-overflow` for an anonymous text `node` (CSS UI 3
§5.2).

`text-overflow` applies to the block container and does **not** inherit, so
the anonymous text node's own computed value is always the initial `clip`.
The governing value is the block container's, read from the text node's parent
(falling back to the node itself). This mirrors the layout coordinator so the
compose pass truncates identically to the measure pass. (Nested inline boxes
are not walked — a documented approximation of the flat text model.)
