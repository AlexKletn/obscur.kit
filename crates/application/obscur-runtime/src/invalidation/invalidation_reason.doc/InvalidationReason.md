Why a change happened — the runtime-level cause — which maps to an
[`InvalidationKind`] and, via the router, to the domain invalidation reasons.

Each variant records the semantic cause of an invalidation as reported to the
runtime; [`kind`](InvalidationReason::kind) classifies it into the shared
[`InvalidationKind`] depth used to flag dirty pipeline phases.
