The event's position in the space of the content the node it landed on
holds: [`screen`](Self::screen) with the `scroll_top` of that node's own
scrolling ancestors — and of the node itself, when it is a scroll container
— added back.

Per target, not per pointer. A `position: fixed` toolbar over a scrolled
list rides no scroll, and handing it a point shifted by a scroll it never
took reported a click tens or hundreds of pixels from where it happened.
Folding through the target's own chain instead reaches the same number as a
whole-frame fold for everything that does sit in the scrolled content — a
row, a caret in a scrolled field, a drag inside a list, which keeps the
same fold even when the pointer wanders out of the scrollport — and a
different one only where the whole-frame fold was wrong. With no target the
whole-frame fold stands in.

Equal to [`screen`](Self::screen) in a frame with no scroll container, so a
document that never scrolls lands byte-identically to one built before
scroll containers existed.
