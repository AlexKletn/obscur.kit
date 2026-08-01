The policy governing a component's error boundary (spec §1.7).

Every component is an error boundary by default, and a component may
customize or explicitly disable that boundary. This small value captures
both choices: whether the boundary is [`enabled`](Self::enabled) at all, and
how many times a contained failure may be retried before the fallback UI is
kept.
