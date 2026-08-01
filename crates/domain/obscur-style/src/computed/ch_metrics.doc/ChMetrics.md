Measures the font metrics the computed-value step needs but cannot derive.

Implemented by whatever layer owns a text shaper (in this workspace, the
runtime's paragraph orchestrator) and handed to the resolver through
[`ResolutionContext::with_metrics`](super::ResolutionContext::with_metrics).

An implementor is a *measurement*, not a policy: it reports what the font says
and returns [`None`] when it cannot, leaving the specified fallback to the
caller. It must not substitute `0.5em` itself — see [`Self::zero_advance`].
