Shapes `content` at `font_size` through the fallback orchestrator and
**wraps** the font-tagged runs to `max_width` into a multi-line, multi-font
[`ParagraphLayout`] (spec §5.10).

Shaping runs **once**, here: [`shape_paragraph`](ParagraphShaper::shape_paragraph)
produces one [`ShapedRun`](obscur_text::shaping::ShapedRun) per script/font run, which
[`ParagraphLayout::wrap`] greedily breaks into lines no wider than
`max_width` (the arranged box width), each line split into contiguous
same-font segments. Because the layout phase measured the box by wrapping
the **same** orchestrator's runs to the **same** width, the wrapped line
count here matches the box height layout produced. A `max_width <= 0` yields
a single line.

`fallback` is the node's resolved font chain (from its computed
`font-family` via [`families_for`], or the default chain when unauthored);
`attributes` (weight + slant, via [`attributes_for`]), `direction` (the
base bidi direction, via [`direction_for`]), and `alignment` (the line
alignment, via [`alignment_for`]) come from the node's computed
`font-weight` / `font-style` / `direction` / `text-align`. The layout phase
measures the box with the identical chain, attributes, direction, and
alignment, so the shaped runs — and therefore the wrapped and justified
lines — agree.
