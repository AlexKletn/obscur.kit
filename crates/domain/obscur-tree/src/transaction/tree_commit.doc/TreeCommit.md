The record of a successfully applied [`TreeTransaction`](super::TreeTransaction).

A `TreeCommit` is produced only after the whole staged batch validated and
was applied. It captures the tree's version immediately before and after
application, plus the concrete effects of the batch. It is read-only.

This is the tree-level primitive that the runtime's UI-op layer (in
`obscur-runtime`, later) will build on: provisional commit/cancel *at the
UI-op level* is coordinated there, using records like this one to reconcile
applied tree effects with the surrounding operation. At the tree level, a
`TreeCommit` simply reports what a committed batch did.
