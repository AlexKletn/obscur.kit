Handles a Ctrl+zoom shortcut, returning `true` when `key` was a zoom chord
(so the caller swallows it instead of forwarding it as text):

- `Ctrl+=` / `Ctrl++` → zoom in (×1.1, capped at 5.0);
- `Ctrl+-` → zoom out (÷1.1, floored at 0.3);
- `Ctrl+0` → reset zoom to 1.0 **and** scroll to the top.

A change re-syncs the viewport + device scale (+ scroll clamp) and requests a
redraw, so the whole showcase grows/shrinks crisply about the new scale.
