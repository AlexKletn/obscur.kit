Whether the cursor has entered the window and not yet reported where.

winit's `CursorEntered` carries no position — a `CursorMoved` always follows it
with the real one — so converting it on the spot would have to reuse the
position the cursor *left* by, entering whatever node happens to sit there and
leaving it again one event later. An author's `on:pointerenter` would fire for a
row the pointer never crossed.

So the entry is remembered and the next move is promoted to the
`PointerEventKind::Enter` instead. One `Enter` per crossing, at a position that
is actually the pointer's.
