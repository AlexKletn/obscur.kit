Whether a node's computed `overflow` makes it a **scroll container** — the
reserved `scroll` / `auto` / `hidden` handles (CSS Overflow 3 §overflow).
`visible` / `clip` / absent are not scroll containers. This is the single-style
twin of [`establishes_scroll_container`](super::scroll::establishes_scroll_container),
which decides the same thing over the whole `ComputedStyles` map.
