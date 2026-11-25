Whether a mutating edit to a field can be flushed with a **compose-only**
frame because it provably cannot change any retained box.

A **borrow bundle** over the two things the answer is read from — the retained
tree and this frame's computed styles — held for one question. It decides
nothing about frames: it answers *is this field box-stable*, and the runtime
turns that into a [`FrameReason`](crate::frame::FrameReason).

# Why the three conditions are one entity

Because they are not three properties of a field, they are one property —
*nothing painted by this frame depends on the width of the text that was
typed* — arrived at from three directions, and it is only true when all three
hold. A wrapping field's height tracks its line count; a centred or RTL field's
run x depends on the box width it is centred in; an `auto`-width field's box
*is* its content. Miss any one and a compose-only frame reuses a retained box
that the typed character has invalidated, and the caret paints where the text
no longer is.

That is also why the conservative direction matters more than the permissive
one: every `false` here costs a full frame the user cannot perceive, and every
wrong `true` is a visual defect. So an unknown — no text leaf, no style, a
percentage width — answers `false`.
