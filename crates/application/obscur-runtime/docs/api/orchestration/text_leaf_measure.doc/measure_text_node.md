Shapes and wraps a text node's `content` to its available width and returns
the wrapped paragraph's border-box size, constrained by `constraints`.

The content is shaped once at `style.font_size` through the multi-font
`paragraph` orchestrator (itemize → resolve → shape) into one or more
font-tagged runs, then wrapped (greedy word wrap) to the finite
`constraints.max.width`, minus the node's own horizontal border/padding
(usually zero for a text node). The returned [`size`](ParagraphLayout::size)
is the widest line's width by the summed line heights, so a long string in a
narrow width yields a taller box; the scene phase re-wraps to this box width
through the **same** orchestrator and so emits exactly the same lines.

`fallback` is the node's font chain resolved from its computed `font-family`
(via [`families_for`](super::families_for)); `attributes` (weight + slant, via [`attributes_for`](super::attributes_for)),
`direction` (the base bidi direction, via [`direction_for`](super::direction_for)), and `alignment`
(the line alignment, via [`alignment_for`](super::alignment_for)) come from the node's computed
`font-weight` / `font-style` / `direction` / `text-align`. The scene phase
resolves the identical chain, attributes, direction, and alignment, so the
measured box and the painted glyphs use the same fonts, reordering, and
justification.
