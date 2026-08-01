Produces this frame's computed styles and returns
`(computed, restyled, delta, wants_animation_frame)`.

`restyle` is the plan's style flag. When it is `false` the phase does not
match or resolve anything: it reuses the retained computed styles and
re-applies the motion overlay, so a motion-only frame still advances its
animated values while touching no sheet.

`author_sheets` is the author half of the cascade the frame resolved before
anything ran (retention included); the UA default is added here, below all
of them, because only [`CascadeSheets`] may put the list together.

`reconcile_roots` is what the reconcile phase could bound its edit to, or
[`None`] when it could not and a full pass is the only correct answer.

`wants_animation_frame` reports that §4.17 S4 started a timeline off this
cascade; the runtime, which owns the scheduler, is what asks for the frame.
