The hover chain one pointer event leaves behind, and whether it differs from
the one before it (spec §2, §4.13 `:hover`).

# Why the device rule belongs to this answer and not to its caller

Because "what is hovered" is not "what was hit": a touchscreen contact does
not leave a persistent hover. The finger has no resting cursor, so when it
lifts the chain must clear even though the lift's hit-test target is the node
it lifted from — while a pen genuinely hovers above the surface and a mouse
stays where it was left, so for both of those the same event leaves the chain
exactly where the hit test put it. One `Up`, three answers, decided by the
device.

Left to the caller that rule is one `if` beside a hit-test result, which is
how it reads as an optimisation and gets "simplified" into the very bug it
prevents: a tapped row that stays lit after the finger is gone, for the rest
of the session or until something else is tapped.
