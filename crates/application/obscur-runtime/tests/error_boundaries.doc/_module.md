What happens when a component's render fails: containment (spec §1.7) and the
report (spec §1.6).

One failure, held fixed — a `step` prop authored as text where an integer is
wanted — and only the boundary *policy* varies. That is the whole design of this
file: any difference in outcome between two tests can only be the policy's
doing, never a different bug.

The **default** policy spends a retry budget: the first failing frame keeps the
subtree and reports `Retried`, the second replaces it with the author's fallback
and reports `BoundaryReplaced`. Both decisions are asserted as an ordered list,
because "it recovered eventually" is not the claim — "it took these decisions,
in this order" is. A **disabled** policy neither retries nor falls back, and
reports `None` exactly *once* rather than re-firing every frame; that
once-and-only-once is the difference between a diagnostic and a log storm.

A disabled boundary with an *enclosing* enabled one is the containment test:
the failure escapes the inner component and is answered by the outer, so the
outer's subtree is what gets replaced, the failing component is unmounted, and
the two diagnostics are anchored to two different nodes — the failing child
reporting no recovery, the boundary that answered reporting the replacement.
With no enclosing boundary at all the failure is uncontained and the last good
output does stand, which is exactly what disabling every boundary above it asked
for; the honest floor is that it is said out loud, once.

A boundary is not a tombstone: when the props decode again the component renders
normally, and recovering is not itself reported as a failure.

Two tests are about the *channel* rather than the decision. One replaces the
bespoke collecting sink with the shipped `ChannelSink`/`DiagnosticChannel` a
host or DevTools backend actually subscribes with, filtered to errors — a
`Diagnostic` built by production code arriving at a real subscriber. The other
removes the sink entirely, because the default `NullDiagnosticSink` must not be
a second code path: the boundary decides and acts identically with nobody
listening.

The one test that stayed in `ui_runtime.rs` is the retry's other half — that a
retry actually *schedules* the frame that spends its budget — which reads the
crate-private pending frame reason. A budget that never gets spent is the stale
UI bug wearing a diagnostic.
