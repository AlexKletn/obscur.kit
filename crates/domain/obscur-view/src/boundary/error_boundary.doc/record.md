Records `failure` as the boundary's current failure and returns whether
this boundary *contains* it.

Returns `true` when the policy is enabled — the failure is contained and
the subtree is now to be replaced by the [`fallback_view`](Self::fallback_view).
Returns `false` when the policy is disabled — the failure is stored (so
it is still reported) but must be propagated to an outer boundary.
