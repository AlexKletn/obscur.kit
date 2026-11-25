The editable's keymap: a [`KeyEvent`] plus the held modifiers, against a
laid-out field, resolved to one [`KeyEdit`] (spec §5.11, §6.7).

A **borrow bundle** over everything the answer is read from — the retained
tree, this frame's computed styles and arranged boxes, the scene's paragraph
orchestrator, the live session and the modifier state — constructed for one
keystroke and dropped with it. It holds no `&mut` anything: rule 5 of this
crate's decomposition keeps the *acting* in the runtime, because applying an
edit means the readonly gate, the `on-input` dispatch, the caret-follow scroll
on both axes and the frame request, and rule 2 forbids this entity from
holding any of them.

# Why the keymap is one entity

Because the map from a keystroke to an edit is **total and exclusive**, and
only a table that sees every key at once can be either. Total: every key the
field does not claim must fall through unchanged, so a bare `a` edits the
field while `Ctrl`+`a` selects it and `Ctrl`+`Shift`+`p` reaches the app's
command router untouched. Exclusive: a key resolved as an edit is *not* also a
shortcut and *not* also an activation, so the decision has to be made once, in
one place, before any of the three paths runs.

The three motions that need the wrapped layout — vertical arrows, `Home` and
`End` — are in here for the same reason rather than beside the caret geometry:
they are not "where is the caret", they are "what does this key mean", and
each one's answer is a *fallback ladder* (no line in that direction clamps to
the text edge; no arranged box at all falls back to the buffer's logical line
motion) that only makes sense as part of the keymap. What the layout supplies
them is one number, which is exactly the narrow interface that marks the
boundary.
