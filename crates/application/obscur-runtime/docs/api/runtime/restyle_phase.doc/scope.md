Classifies this frame's style phase into a per-node dirty set, or `None`
when the frame cannot prove which nodes a full pass would change (§3.4
phase 5, §4.3).

§4.3 asks that recalculation start "at the nearest affected context" and
that ad-hoc dirty flags not be the model. This is where a frame stops
asking *whether* to restyle and starts asking *what*: it returns the typed
[`StyleInvalidation`] accumulator that
[`StyleMatchingCoordinator::rematch_into`] and
[`StyleCoordinator::resolve_incremental`] then consume.

# The one case it narrows, and why only that one

The retained inputs and styles can be reused only for nodes a full pass
would recompute identically, which needs every input of the match + style
phase to be unchanged bar one whose blast radius is known. Those inputs are
the tree, the sheets, the media environment and the interaction state; this
requires the first three to be unchanged ([`StyleKey`], plus the frame's own
`reconciled` result for the tree) and derives the dirty set from the fourth
via [`RestyleScope`]. Every other cause of a restyle — a new stylesheet, a
resize, a tree edit — answers `None` and restyles in full, which is what the
runtime did for all of them before.

# Why a reconcile is no longer an automatic full pass

It used to be: any tree edit answered `None`. But the reactive path — a
builder that returns a fresh `View` every frame — classifies *every* frame as
structural, so the incremental path was unreachable from a real windowed host
and a 300-node tree restyled in full at 60Hz. The reconciler already reports
exactly which nodes it touched (it compares by value: identical text and
props produce no op at all), so those ids become restyle roots here and are
expanded like any other. Only a change that cannot be bounded — a removal,
whose node is gone before its shifted siblings can be reached from it, or a
component sub-reconcile this frame never sees — widens back to everything.
