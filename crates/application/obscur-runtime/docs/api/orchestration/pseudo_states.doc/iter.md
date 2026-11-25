Iterates the `(node, state)` pairs of every node carrying a non-empty
interaction state.

Because the map is canonical (see [`set`](Self::set)), a node absent from
this iteration reports [`PseudoState::empty`]. That is what lets two maps
be diffed by walking both sides' entries rather than the whole tree — see
[`RestyleScope::changed_states`](crate::invalidation::RestyleScope::changed_states).
