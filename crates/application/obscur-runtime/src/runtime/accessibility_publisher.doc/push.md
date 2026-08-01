Pushes `focused` and `snapshot` to `bridge`, and records `focused` as
published.

Focus goes first because it is not carried *by* the snapshot as far as an
adapter is concerned: the semantic nodes record which one is
[`focused`](obscur_accessibility::semantic::SemanticState::focused), but
an adapter converts a tree against the focus target it was last *told*,
so a snapshot pushed alone would arrive with the platform's focus still
on whatever it was — for AccessKit, the root.

The record advances for whichever bridge was pushed to, which matters
where a host drives a second bridge of its own alongside an installed
one: that push satisfies the gate, and the installed bridge then misses
the focus move on the next frame unless the tree changed too. The two
routes are documented as alternatives ("a bridge of its own **instead**")
and one record is what makes them so.
