Scroll-into-view for the **caret** of the focused editable (CSS Overflow 3
§3.1): how far a field must scroll so the caret the user is moving stays
inside the field's scrollport.

A **borrow bundle** over what the answer is measured from — the tree, this
frame's computed styles and arranged boxes, and the scene's paragraph
orchestrator — held for one recomputation. Rule 2 of this crate's
decomposition keeps the *writes* out: both methods **report** an offset and
the runtime seats it, because the two offsets live in two different owners
(the session's `scroll_left`, the scroll map's `scroll_top`) and neither is
this entity's to hold.

# Why the two axes are one entity

They are the same clamp, twice, and they are only correct while they agree.
Both keep a caret extent inside `[offset, offset + extent]`, snap to whichever
edge it crossed, and then clamp to `0 ..= max(0, content − scrollport)`; both
measure the caret from the **live session buffer** through the same shaper
compose paints with, so the offset and the painted glyphs cannot disagree; and
both take their scrollport from the field's border box minus its own
padding + border. Written apart, the failure is the one this campaign's ledger
already records for the editing tier: a path that re-derives one axis and
forgets the other leaves a field scrolled for content it no longer holds.

Which axis applies is a property of the field, not of the caller: a
non-wrapping single-line field scrolls horizontally and never vertically; a
fixed-height `<textarea>` whose overflow scrolls does the reverse. Each method
answers [`None`] for a field that is not its case, so a caller can ask both.
