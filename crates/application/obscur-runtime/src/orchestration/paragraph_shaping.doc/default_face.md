The default fallback face requests resolve to when the resolver covers no
font for a run. The mock shaper is font-agnostic (it needs no real
metrics), so a generic monospace face suffices as the default; a real
pipeline would resolve this per node via a `FontProvider`.
