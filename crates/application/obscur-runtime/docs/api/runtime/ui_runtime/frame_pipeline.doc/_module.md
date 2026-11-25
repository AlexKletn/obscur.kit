The frame itself: the sequence that turns a classified [`FramePlan`] into a
[`FrameResult`], and the two readings the sequence takes of the runtime's own
state on the way through.

Almost nothing here computes anything. Every phase this file names is already an
entity in its own file — [`ReconcilePhase`], [`StylePhase`], [`ArrangePhase`],
[`ComposePhase`], with [`LayoutClassifier`] refining the plan between the last
two — and what is left after them is the **ordering**, which is a thing in its
own right: dispatch before the plan is chosen, resource results committed before
the plan is chosen, images resolved before layout rather than at paint, the
controlled-input reseed after reconcile and before measure, accessibility
published after both reconcile and dispatch have settled. Each of those is a
constraint someone can violate by moving three lines, and the comments beside
them are the only record of why they cannot.

The sequence is therefore kept **whole**. It was not split into per-phase methods
because the phases do not have independent semantics left to give: a `style_step`
would take seven arguments and return four values, and its body would be the
`StylePhase` constructor it already is. What did come out is the one question the
sequence asks before it starts — which phases this frame owes — and that is
`frame_classification.rs`.

[`FramePlan`]: crate::runtime::FramePlan
[`FrameResult`]: crate::frame::FrameResult
[`ReconcilePhase`]: crate::runtime::ReconcilePhase
[`StylePhase`]: crate::runtime::StylePhase
[`ArrangePhase`]: crate::runtime::ArrangePhase
[`ComposePhase`]: crate::runtime::ComposePhase
[`LayoutClassifier`]: crate::runtime::LayoutClassifier
