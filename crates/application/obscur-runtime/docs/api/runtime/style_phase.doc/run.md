Produces this frame's computed styles and returns
`(computed, restyled, delta)`.

`restyle` is the plan's style flag. When it is `false` the phase does not
match or resolve anything: it reuses the retained computed styles and
re-applies the motion overlay, so a motion-only frame still advances its
animated values while touching no sheet.

`author_sheets` is the author half of the cascade the frame resolved before
anything ran (retention included); the UA default is added here, below all
of them, because only [`CascadeSheets`] may put the list together.

`reconcile_roots` is what the reconcile phase could bound its edit to, or
[`None`] when it could not and a full pass is the only correct answer.

The phase starts any CSS timelines triggered by this cascade. The runtime
queues another frame while timelines remain active.

`reconcile_roots` also bounds the §4.16 generated-content pass, which is scoped
by the same dirty set the cascade was and keeps the previous frame's answer for
every node outside it.
