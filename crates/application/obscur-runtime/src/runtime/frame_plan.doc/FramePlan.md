The pipeline phases a single frame must run, classified from its
[`FrameReason`] via the [`InvalidationRouter`] (spec §3.4 phase 5, §4.3).

Each frame the runtime builds a `FramePlan` and gates its phases on it: a
phase whose inputs are unchanged is skipped and reuses the previous frame's
retained output. The classification is conservative — when a phase cannot be
proven unnecessary it is run — so correctness is preserved while clearly
redundant work (e.g. re-reconciling on a hover-only frame) is elided.

The mapping from reason to invalidation cause is:

| [`FrameReason`]        | [`InvalidationReason`]   | reconcile |
|------------------------|--------------------------|-----------|
| `UiCommit`             | `TreeChanged`            | yes       |
| `ExternalRender`       | `TreeChanged`            | yes       |
| `ResourceCompletion`   | `TreeChanged`            | yes       |
| `WindowChange`         | `EnvironmentChanged`     | no        |
| `Input`                | `PseudoStateChanged`     | no        |
| `TextEdit`             | *(compose-only)*         | no        |
| `Animation`            | `StyleChanged`          | no        |
| `DiagnosticsCapture`   | *(none — idle)*          | no        |

[`TextEdit`](FrameReason::TextEdit) bypasses the router entirely and maps
straight to a [`compose_only`](Self::compose_only) plan (compose runs; style,
layout and reconcile are all skipped): a same-size text edit re-shapes and
repaints the field from the live editing session without disturbing any
retained box, so it stays pixel-identical to the full frame while eliding the
per-keystroke restyle + relayout.

The style/layout/compose flags are then read straight off the router's
dirty-phase flags, so the router — previously orphaned — is the live source
of truth for what a frame recalculates.
