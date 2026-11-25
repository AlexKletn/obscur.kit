Whether this node is a slot outlet, named or not (spec §2.12).

The single predicate for "this node is an outlet". [`project_slots`]
substitutes every node it answers `true` for; the reconciler refuses to
materialize one, so a node it answers `true` for never becomes retained.
Both must mean the same thing by construction, which is why there is one of
these rather than a `component == SLOT_OUTLET` test in each.
