§5.3's custom rendering region, as a bounded region of an ordinary display list.

The spec says "the portable Canvas API uses the common Paint model", and it
means it literally: there is no second drawing vocabulary here.
[`painter`](CanvasRegion::painter) hands back the same `DisplayListBuilder`
every other node paints through. What a region adds is the two things an
application painter cannot be trusted to get right about a list it shares.

**Its own coordinates.** A translation to the laid-out box means a painter
draws at `(0, 0)` and lands at its top-left, wherever layout put it — and
[`at`](CanvasRegion::at) reads the normalized coordinates §4.5 promises, so a
ring at half the width need not be told the width.

**Containment.** A clip to its own box, and a close that unwinds whatever the
painter left open. An unbalanced painter would not corrupt itself; it would
swallow every node painted after it into a bracket that was never meant for
them. [`close`](CanvasRegion::close) reports whether it had to do that, so a
host can tell the author rather than silently repairing it forever.
