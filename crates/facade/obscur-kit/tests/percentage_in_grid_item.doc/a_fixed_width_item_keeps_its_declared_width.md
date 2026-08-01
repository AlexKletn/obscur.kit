The audit's third row: a `width: 50px` item keeps its declared 50.

This was the audit's other defect (2-4 (b)), and it used to read 211 — the
whole track. `specs/w3c/css-align-3.txt:1377` scopes `stretch` to a box "When
the box's computed width/height (as appropriate to the axis) **is auto**",
falling back to `flex-start` otherwise, so an item with a width of its own is
never stretched: it keeps that width and sits at the start of its grid area.
The child's 25 is half of the declared 50 and never moved — this row's child
was never the problem, which is exactly why the two defects had to be pinned
apart.
