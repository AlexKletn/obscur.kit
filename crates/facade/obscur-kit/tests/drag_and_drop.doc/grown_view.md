The same view with an extra node under `.sink`, so that "did the drop's
transaction classify a reconcile" is observable.

Asserting on a pending frame would not do it: the drag requests frames for its
own visuals, so one is pending either way. Asserting on `reconciled()` against
an identical view would not either — a reconcile over an unchanged view commits
empty and reports `false`. Only adopting a node the tree did not have can tell
the two apart.
