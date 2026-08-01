The `overflow` property (Keyword-valued: `visible` (initial) / `hidden` /
`clip` / `scroll` / `auto`; NOT inherited). See the reserved `OVERFLOW_*`
handles in [`crate::property::layout_keyword`]. A value other than
`visible` clips the element's subtree to its box; `scroll` / `auto` clip
like `hidden` (no scrollbars are synthesised yet — a documented
simplification). It changes no layout, only the scene coordinator's clip.
