A component that renders the word "healthy" and takes
[`CounterProps`](super::CounterProps) — which is the entire trick.

Sharing the props type means the *failure* is fixed: a `step` authored as text
where an integer is wanted cannot decode, for this component or for the counter.
What varies between tests is only the
[`ErrorBoundaryPolicy`](obscur_view::boundary::ErrorBoundaryPolicy) and the
fallback view handed to [`registration`](Self::registration) — default,
retries-exhausted, or disabled — so a difference in outcome can only be the
policy's doing.

[`text_view`](Self::text_view) is here rather than loose beside it because every
fallback these tests hand in is a bare line of text, and comparing the rendered
child against a string is how "which subtree is on screen" gets asserted.
