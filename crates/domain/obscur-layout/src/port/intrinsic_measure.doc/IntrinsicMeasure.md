The narrow **port** layout calls to measure content it cannot size itself —
text runs and replaced elements such as images or canvases (spec §2.3).

Layout never shapes text or decodes images; instead it asks a provider
behind this port. That keeps the layout engine independent of text and
image-decoding internals: the real text engine lives in `obscur-text` (added
in a later phase) and provides an adapter implementing this trait, while a
deterministic mock — [`MockIntrinsicMeasure`](super::MockIntrinsicMeasure) —
serves the Phase-1 headless slice (spec §5.13).

Implementations **must** be deterministic and side-effect-free: the same
`(node, constraints, env)` inputs must always yield the same size, and a
measurement must never mutate observable state. This lets layout cache and
re-run measurement freely.
