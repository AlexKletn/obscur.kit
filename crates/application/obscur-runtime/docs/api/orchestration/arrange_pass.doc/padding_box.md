The **padding box** of a node — its absolute border-box `border_box` inset by
its `border` edges — which is the containing block a positioned node
establishes for its absolutely-positioned descendants (spec §4.1).

Extents are clamped non-negative so a box smaller than its borders yields a
zero-extent (never negative) containing block. With no authored border-width,
`border` is zero and the padding box equals the border box.
