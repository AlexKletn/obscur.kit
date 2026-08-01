The outcome of a recovery decision for a failed unit of work (spec §1.7).

A recovery decision follows the spec's *contain → (retry | fallback) |
propagate* rule. This value records which of those three branches was taken
so the runtime can act on it — rerun the unit, keep the fallback UI, or hand
the failure to an enclosing boundary — and report the choice to diagnostics
via [`recovery_action`](Self::recovery_action).
