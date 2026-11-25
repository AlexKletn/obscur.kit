Emits a text node's paint into `node_paint`, the node's own paint unit: one
text run **per wrapped line** at its arranged box, plus — for the focused
editable — the selection highlight behind the glyphs, the IME preedit underline
and the caret on top of them.

A node is either a text node or an element with a background, so this never
double-emits with the background fill. The content is wrapped to the box width
the layout phase measured with — so the emitted line count matches the box
height — and each line is drawn at `(origin.x + x_offset, origin.y + top)` with
its glyphs on that line's baseline. A text node with no arranged box is
skipped.

`editing` is the runtime's active text-editing session, or [`None`] when no
editable is focused; with [`None`] the retained content paints with no caret
and no highlight, exactly as before editing existed (the byte-identical
regression path).
