Returns whether an external `edit` is valid against the composition (spec
§5.12 "external edits validate the composition range").

While [`Composing`](CompositionState::Composing) an external edit is
invalid when it overlaps the composition [`anchor`](Self::anchor) — an
edit inside the live composition range would corrupt it — so this returns
`false` for an overlapping edit and `true` otherwise. When the composition
is no longer composing there is nothing to protect, so it is always
`true`.

Overlap is tested with [`TextRange::intersects`], which treats
endpoint-touching ranges (e.g. `0..2` and `2..4`) and empty ranges as
non-overlapping.
