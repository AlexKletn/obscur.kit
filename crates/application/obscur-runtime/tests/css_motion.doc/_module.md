CSS transitions and `@keyframes`, driven through the runtime's motion
orchestration (spec §4.17, css-animations-1 §3.9).

Six tests over one pipeline: the cascade resolves a value, motion overlays a
sampled one, layout arranges the result and paint composes it. Every assertion
here reads the *arranged box*, never the computed style — a motion overlay is a
paint-time concern and must not mutate computed style, which the spec suite
pins separately and which four deleted unit tests once asserted the opposite of.

A transition fires on a **change** and only on a change, interpolates across its
duration and is pruned when it finishes. A keyframe rule registers once per match
rather than once per frame. A delayed animation shows its first keyframe during
the delay under a `backwards` fill and shows the cascade value under `none` —
the one case where "nothing happened yet" and "the wrong thing happened" look
identical from the outside.
