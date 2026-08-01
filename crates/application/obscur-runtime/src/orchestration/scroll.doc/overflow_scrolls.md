Whether a node's computed `overflow` makes it a **scroll container** — the
reserved `scroll` or `auto` handles (CSS Overflow 3 §overflow). `visible` /
`hidden` / `clip` / absent are not scroll containers (they never own a scroll
position, even though `hidden` / `clip` still clip).
