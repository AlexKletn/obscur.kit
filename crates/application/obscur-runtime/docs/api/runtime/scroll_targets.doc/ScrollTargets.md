Every scroll container this frame has, and the three questions the rest of the
runtime asks about them (CSS Overflow 3 §scroll-container).

A **borrow bundle** over the retained frame — tree, computed styles, arranged
boxes — plus the live offsets, held for one question and released with it. It
resolves containers; it never moves one. Every method here is a read, and the
two writers ([`UiRuntime::scroll_by`](super::UiRuntime::scroll_by) and the
caret-follow) go through [`ScrollOffsets`], which owns that write.

# Why the three questions are one entity

Because they must agree about **which boxes are scroll containers and which of
those the user may scroll**, and those are two different predicates that are
easy to confuse. A box with `overflow: hidden` *is* a scroll container in the
CSS sense — it clips, and it can be scrolled programmatically — but the wheel
must pass straight through it, or a row that clips a long name eats the scroll
meant for the list it sits in. So [`scrollable_at`](Self::scrollable_at) and
[`enclosing_scrollable`](Self::enclosing_scrollable) filter on
`overflow_scrolls` while [`fold_point`](Self::fold_point) does **not** — a
`hidden` container still translates its content, so a click inside one must
still be un-translated or it hit-tests the wrong row. Getting that split wrong
in one place and right in another is invisible until a user scrolls the wrong
box or clicks the wrong row, and both are silent.

[`paint`](Self::paint) carries the same distinction outward: every container
reports its offset, but only a scrolling one reports a scrollbar, because
advertising one would promise a scroll the user cannot perform.
