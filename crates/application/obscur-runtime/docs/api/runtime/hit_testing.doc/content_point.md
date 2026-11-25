`screen` in the space of the content `node` HOLDS, rather than the space its
own box lives in: [`folded_point`](Self::folded_point) plus `node`'s own
`scroll_top` when it is itself a scroll container.

The difference is one element deep and it matters twice. A pointer landing
on a scroll container's own box — the strip beside the last line of a
scrolled `<textarea>`, say — addresses that container's *content*, so the
caret seat and every handler reading the event's position want the scrolled
coordinate; a pointer landing on the container's *border* wants the same
number, because a scrollport and its content are one space to everything
downstream. But a `position: fixed` toolbar merely overlapping a scrollport
holds no scrolled content of its own and rides no ancestor's, so it gets
`screen` unchanged — which is the whole point of asking per node instead of
folding the pointer once for the frame.

Use [`folded_point`](Self::folded_point), not this, to decide whether a
point is inside a box: a container's own box does not move when its content
scrolls.
