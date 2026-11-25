Records that this render depends on the observable `source` (spec §3.1).

This is the whole of dependency tracking from the author's side: read a
source, say so. The runtime then invalidates this component when — and
only when — that source advances. A component that tracks nothing is never
invalidated by a state change; a component that tracks `A` is untouched
when `B` changes.

Dependencies are re-collected on every render, so a component that reads a
source only on some branch depends on it only when it actually did. The
generation is captured here, not by the caller, so a source that advances
*between* this render and the next is correctly seen as changed.
