Applies the component error-boundary policy (spec §1.7).

Every component is an error boundary by default: a failure in view
generation, reconciliation, or a local lifecycle hook is *contained* within
the component subtree, may be *retried* according to policy, and otherwise
the subtree is replaced with fallback UI — unless the boundary is disabled,
in which case the failure *propagates* to the nearest enclosing boundary.

This coordinator is stateless: it sequences the §1.7
*contain → (retry | fallback) | propagate* decision, delegating the
contain/retry policy itself to `obscur-view`'s [`ErrorBoundary`]. The runtime
only orders the decision and (later) reports it to diagnostics and reruns or
replaces the subtree; it does not reimplement the boundary here.
