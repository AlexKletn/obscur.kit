The [`LayoutObserver`] — watches a node's full layout box (spec §4.4).

Geometry observers run *after* layout commit, are coalesced, and report the
net change to their watched property; the callback dispatch and transaction
scheduling are the runtime's job. This type models only the observer state
and the pure change detection: given the newly-committed [`LayoutBox`], it
reports whether the box changed since the last observation.
