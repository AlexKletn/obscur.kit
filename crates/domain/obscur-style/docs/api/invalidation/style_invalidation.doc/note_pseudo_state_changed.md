Invalidates `node` if `dep` shows its matching depends on a pseudo-state
that `changed` toggled (§4.14).

Consults [`StyleDependency::affected_by_pseudo_change`] and records
[`StyleInvalidationReason::PseudoStateChanged`] on a match. Returns whether
`node` was invalidated by this call.
