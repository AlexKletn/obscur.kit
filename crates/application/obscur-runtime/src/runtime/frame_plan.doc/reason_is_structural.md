Whether `reason` can change the retained tree's structure and therefore
requires a reconcile pass.

Only a state commit, an external render request, or a resource completion
can alter the desired tree; input, animation, window and diagnostics
frames leave the structure untouched, so reconcile (an idempotent no-op
for them) is skipped.
