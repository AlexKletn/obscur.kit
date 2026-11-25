Keeping the caret inside the field it is in (CSS Overflow 3 §3.1
scroll-into-view), on both axes.

[`CaretScroll`](crate::runtime::CaretScroll) computes *where a field would have
to be scrolled* for the caret to be visible; it holds neither the session nor
the scroll map, so it can only report. These two are the write half, and they
are a pair rather than one method because the two axes are seated in different
places: a single-line field's horizontal offset belongs to the **session**,
since it is a property of that field's own text run, while a `<textarea>`'s
vertical offset belongs to the **scroll map**, since a multiline field is an
ordinary overflow scroll container that a wheel or a scrollbar can move too.
Seating the vertical one anywhere else would give a textarea two `scroll_top`s
that disagree.

Both are no-ops when there is no session, and both are no-ops when the content
fits — a caret that is already inside the content box moves nothing.

**Reported, not changed** (see `docs/runtime-decomposition.md`): the two callers
do not treat the axes alike. `after_edit` re-derives both; the controlled-input
reseed in `text_editing.rs` re-derives only the horizontal one, so a controlled
`<textarea>` whose value is externally rewritten shorter keeps a `scroll_top`
computed for the old, longer content. That is a behaviour change with its own
test, not something to slip into a move.
