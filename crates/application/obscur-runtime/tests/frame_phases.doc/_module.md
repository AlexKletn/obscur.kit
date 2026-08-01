Which phases a frame actually ran, and why it was allowed to skip the rest
(spec §3.4 p5, §4.3).

Four tests about one refinement: a frame's plan is classified from its *reason*,
which is a conservative over-estimate, and the phases it really runs are then
narrowed by what changed. A `:hover` rule that moves `background-color` cannot
have moved a box, so the layout phase is provably skippable and the retained
boxes carry over untouched; a `:hover` rule that moves `width` cannot skip it.
The same sheet, the same view, the same gesture — only the property differs, and
that is the whole subject.

Each pair is then run twice, because the win has to bite where hosts actually
live: `run_due_frame` with an explicit view, and `run_due_frame_reactive`, which
classifies **every** frame as a structural `UiCommit`. Gating on the plan's
`reconcile()` flag would have made the optimization dead code in real windows;
the gate is the reconcile's actual *result*, and the text-change test is the
safety side of that — an unchanged rebuild commits empty, a rebuild whose text
moved does not, and text is not a computed style.

The three tests that also assert `FramePlan::for_frame(…)` directly stayed in
`ui_runtime.rs`: `FramePlan` is crate-private, and widening it to serve a test is
what this campaign forbids.
