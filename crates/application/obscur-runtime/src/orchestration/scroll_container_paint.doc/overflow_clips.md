Whether a node's computed `overflow` clips its subtree to its box.

Reads both axis longhands through
[`OverflowAxes`](obscur_style::computed::OverflowAxes), which applies
css-overflow-3 §3's pairing rule first. Every value except `visible` clips
(`hidden` / `clip` / `scroll` / `auto`), and one clipping axis clips the whole
box: the kit's clip is a rect, not an axis. An absent property is the CSS
initial `visible` — no clip, byte-for-byte identical to before overflow existed.
