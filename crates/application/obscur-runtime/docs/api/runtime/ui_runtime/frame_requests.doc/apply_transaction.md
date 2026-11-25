Applies a committed [`UiTransaction`](crate::transaction::UiTransaction):
classifies the phases its operations require and requests the frame that
will run them (spec §3.4 phases 4–5).

This is what a [`UiOperation`](crate::transaction::UiOperation) *does*.
Without it the vocabulary is a validation gate and a record: operations are
normalized and checked, and then nothing consults them. Here they decide
the frame's [`FramePlan`], so a transaction that only invalidated a style
produces a frame that restyles **without reconciling** — the tree cannot
have changed, because no operation said it did.

# An empty commit costs nothing

Committing no operations stages an idle plan and requests no frame. That is
what makes [`ProvisionalTransaction`] genuinely cheap to abandon: cancelling
never reaches this method, so a drag that goes nowhere costs zero frames
rather than one that recomputes nothing.

Returns the scheduler's [`BackpressureDecision`], or [`None`] when the
commit was empty and no frame was requested.

[`ProvisionalTransaction`]: crate::transaction::ProvisionalTransaction
