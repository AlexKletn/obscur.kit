Mirror of `obscur_view::view::SLOT` — a child's `slot="…"` assignment, naming
the slot of the enclosing component it is projected into (spec §2.12).

Reserved rather than hashed because the runtime reads it: the reconciler keeps
the authored children of a node that carries one, and `project_slots` matches
them against the outlets the component's own render declares. A hashed symbol
would have made this a name only the front ends agreed on, and the tier that
has to act on it lives below them and cannot call this function.

Not a style attribute: a slot assignment is structure, so it must not affect
matching or specificity. It has a namespace of its own (`0x0000_03xx`).
