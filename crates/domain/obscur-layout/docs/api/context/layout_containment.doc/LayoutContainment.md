Declares whether a [`LayoutContext`](super::LayoutContext) *contains* layout
and/or size effects so that changes inside it do not cross its boundary
(spec §4.3; analogous to CSS `contain`).

A containment boundary stops **upward** invalidation propagation: when a
context contains layout, changes to its internal layout cannot escape to
affect ancestors, so incremental recalculation need not propagate past it.
