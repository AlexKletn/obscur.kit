Classifies the phases a **committed transaction** requires, from its
operations rather than from a frame reason (spec §3.4 phases 4–5).

# Why this is better information than a [`FrameReason`]

[`for_frame`](Self::for_frame) derives the plan from the *category* of a
frame, which is a proxy: `UiCommit` means "something committed", so it
conservatively runs everything. A committed transaction lists what actually
changed, so the plan comes from [`UiOperation::invalidation`]. A transaction
of nothing but [`InvalidateStyle`](UiOperation::InvalidateStyle) restyles
**without reconciling**, because no operation in it claims the tree changed.

# It decides WHICH PHASES run, not what they do

The gain is one bit per phase and no more. Every operation's `node` payload
is **discarded here** — `Rerender { node }` and `ReplaceRoot` collapse to the
same `reconcile: true`, and a reconciling frame then reconciles the whole
desired [`View`]. So this narrows a frame from four phases to fewer; it does
not narrow any phase to a subtree.

That is not a detail to fix here — narrowing a phase to a subtree is a change
to the phase, not to its classification. Where that has happened, it is driven
from the change itself rather than from this plan: the style phase now takes a
dirty set (`StyleCoordinator::resolve_incremental` consumes a
`StyleInvalidation` that `RestyleScope` derives from the per-node interaction
state), but it derives that set in `UiRuntime::restyle_scope`, which reads the
live [`PseudoStates`](crate::orchestration::PseudoStates) map — not the `node`
payloads here. Layout takes no dirty set at all.

So these payloads remain written and never read. Routing them into the dirty
set an incremental pass consumes would mean an
[`InvalidateStyle`](UiOperation::InvalidateStyle) restyling one subtree instead
of the tree; that is the natural next step and is not taken here.

An **empty** transaction yields an [idle](Self::is_idle) plan: nothing was
asked for, so nothing runs. That is what makes a cancelled provisional
transaction free — it stages operations, commits none, and no phase is
classified.

Reconcile is flagged by the operations that can change tree structure
([`ReplaceRoot`](UiOperation::ReplaceRoot) /
[`Rerender`](UiOperation::Rerender)); the style/layout/compose flags come
off the router, exactly as in [`for_frame`].
