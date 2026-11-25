Whether two axis-aligned rectangles overlap in any area.

Half-open on every edge, matching [`Rect::contains`]: a box that merely touches
the viewport's edge shows no pixel and so does not count as visible. A
zero-extent box never intersects anything.
