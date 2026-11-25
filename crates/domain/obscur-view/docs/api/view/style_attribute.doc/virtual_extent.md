A realized virtual-list spacer's extent along the scroll axis, in logical
pixels — the space the rows outside the window would have taken.

Not a CSS input: it changes with the scroll position and the model, so it
cannot be authored. The runtime reads it to size the spacer, which is what
keeps a virtualized container's content extent (and therefore its scrollbar)
true while only the window exists as nodes.
Stored as whole logical pixels: the value is a span of hundreds to hundreds
of thousands of them, so sub-pixel precision is meaningless, and the
attribute set derives `Eq`.
