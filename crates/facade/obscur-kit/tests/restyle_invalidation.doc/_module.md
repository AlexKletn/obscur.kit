Incremental-restyle invalidation must survive a **structural** change.

A reconcile no longer forces a full restyle: the style phase derives its dirty
set from the reconciliation ops. That is only correct if a structural edit
invalidates the siblings whose match it moved — and the structural
pseudo-classes count from **both** ends (`:last-child`, `:nth-last-child`,
`:only-child`, `:nth-last-of-type`), so an insert at the end re-matches the
sibling *before* it.

This is a regression test for exactly that: the first version of the
reconcile-derived scope expanded from the edited node, whose reach is its own
subtree plus the siblings that FOLLOW it, and so left a stale `:last-child`
behind. Nothing in the spec suite caught it.
