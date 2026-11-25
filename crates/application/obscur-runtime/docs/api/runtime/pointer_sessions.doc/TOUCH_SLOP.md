How far a direct contact must travel from where it went down before it is a
pan rather than a tap, in logical px (spec §6.16 `Pan`).

Larger than [`MULTI_CLICK_RADIUS`] on purpose: a fingertip covers millimetres
of screen and rolls as it presses, so the movement that still counts as
"held still" is much wider than a mouse's. Too small and every tap scrolls the
list a little; too large and the list feels stuck before it starts moving.
