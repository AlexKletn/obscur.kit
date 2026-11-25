A runtime showing this field seeded with `seed`, focused by a press inside it,
with the caret at the end of the seed and an `on-input` handler mirroring the
resulting text into the returned string.

The mirror is what proves an edit reached the app: it is written on every
mutating edit and never on a caret move.

# Panics

If the press did not seat focus — which would mean the geometry moved out from
under the coordinate, and every test built on this fixture is measuring the
wrong thing.
