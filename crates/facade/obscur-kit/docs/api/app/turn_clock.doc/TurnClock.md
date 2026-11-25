The wall clock the desktop-surface loop hands to `advance_motion`.

One mark, moved every time it is read. That is the whole type, and it exists
because the loop it serves got exactly that wrong: it advanced motion on every
turn from a mark it moved only on the turns that ended in a frame, so a turn
that found no swapchain image — a bounded wait that timed out, an input burst,
a resource that landed — handed the motion clock the *whole* interval since the
last frame, and then the next turn handed it that interval again plus its own.
Animations, flings and the canvas clock ran fast in proportion to how many turns
a frame took, which is precisely the loaded case where they should not.

Time is neither lost nor double-counted here: consecutive readings partition the
interval between them. Motion is a linear accumulator, so splitting a gap across
the turns it spans is the same total and a closer sampling of the curve.
