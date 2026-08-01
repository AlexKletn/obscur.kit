Handles a Ctrl+zoom shortcut, returning `true` when `key` was a zoom chord
(so the caller swallows it): `Ctrl+=`/`Ctrl++` zoom in (×1.1, cap 5.0),
`Ctrl+-` zoom out (÷1.1, floor 0.3), `Ctrl+0` reset zoom **and** scroll. A
change re-syncs the viewport + device scale (+ scroll clamp) and requests a
redraw.
