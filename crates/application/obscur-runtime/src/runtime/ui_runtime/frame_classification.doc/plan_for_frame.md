Chooses the [`FramePlan`] this frame runs — which of reconcile, style, layout and
compose it owes (spec §3.4 phase 5).

Four things get a say, and the precedence between them is the whole of this
method. A plan staged by [`apply_transaction`](Self::apply_transaction) beats the
one derived from the frame's [`FrameReason`], because it was classified from the
operations that actually changed rather than from the frame's category. The first
frame over an empty tree beats both, because there is nothing retained to reuse.
And the plan the dispatch phase folded in, plus the one a landed resource result
implies, are **merged** into the result rather than replacing it — a hover that
restyles and a commit that reconciles are both owed by this one frame.

`dispatched` and `landed` are reported by the caller rather than re-derived here:
both phases have already run by the time the plan is chosen, and that ordering is
the pipeline's to state. It is also what lets input dispatched *this* frame be
served by *this* frame instead of asking for another.
