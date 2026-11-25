The `pointer-events` property (Keyword-valued: `auto` (initial) / `none`;
INHERITED). See the reserved `POINTER_EVENTS_*` handles in
[`crate::property::layout_keyword`].

`none` makes the element transparent to the pointer: a hit test at a point
inside its box falls through to whatever is behind it (spec §4.7, "hit
testing accounts for … pointer-event policy"). It inherits, so the whole
subtree is transparent as well, and a descendant declaring `auto` becomes
hittable again — the overlay-with-a-live-button pattern.

Inheritance is not a convenience here: it is what CSS specifies
(css-ui / SVG 1.1 §16.6 both give `pointer-events` "Inherited: yes"), and it
is also what makes the subtree rule fall out of the ordinary cascade instead
of needing an ancestor walk at hit-test time.

The SVG-only values (`visiblePainted`, `visibleFill`, `visibleStroke`,
`visible`, `painted`, `fill`, `stroke`, `all`) are deliberately absent: they
condition hit testing on fill/stroke geometry and on `visibility`, neither of
which this renderer models.

Paint-only in the invalidation sense: changing it moves no box, so a frame
that only changed it reuses the retained layout.
