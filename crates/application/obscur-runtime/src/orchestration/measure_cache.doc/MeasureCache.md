A memoization of [`measure`](super::measure_pass::measure): a `(node, constraints)` pair is sized **once**,
collapsing the redundant re-measurement (and text reshaping) the
measure/arrange walk would otherwise do — `arrange(parent)` measures a subtree,
then `arrange(child)` re-measures it under the same constraints, and so on down
the depth. Keyed by the node and the constraint's `f32` bit patterns — the four
range bounds *and* the percentage basis (exact, no hashing/collisions).
`RefCell` allows the short borrow for a
lookup/insert to nest inside the recursive algorithm call that happens between
them.

# It is the §4.2 "measure cache", and it lives longer than a pass

[`layout_editing`](super::LayoutCoordinator::layout_editing) builds a fresh one per
call, so it memoizes within one frame only. A caller that keeps one across
frames and hands it to
[`layout_invalidated`](super::LayoutCoordinator::layout_invalidated) instead gets the
§4.3 mechanism: that entry point drops exactly the entries a
[`LayoutInvalidation`](obscur_layout::invalidation::LayoutInvalidation) can have changed and every surviving entry is served
without re-measuring. Reuse across frames is sound **only** through
`layout_invalidated`, whose doc states the preconditions the caller owes.

[`measured`](Self::measured) / [`measured_count`](Self::measured_count) report
what the **last** pass actually computed — the frame's real measure cost, and
the only way to tell a served entry from a recomputed one.
