Why a node must be restyled: the typed [`StyleInvalidationReason`] (§4.3).

Style invalidation in Obscur Kit 2 is dependency-driven with *typed*
invalidation reasons, not a scattering of ad-hoc dirty booleans (§4.3). Each
reason records the kind of input change that forced a node back into the
restyle set, so the runtime can reason about, log and prioritise pending work
deterministically.
