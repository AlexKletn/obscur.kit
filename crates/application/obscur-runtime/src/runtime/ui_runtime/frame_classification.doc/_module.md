The question every frame asks before it runs a phase: **what does this frame
owe?**

Four independent sources answer it and they do not agree by construction — the
frame's [`FrameReason`], a [`FramePlan`] staged by a committed transaction, a
plan the dispatch phase folded in while it was running, and a resource result
that landed on the channel this frame. This file is the precedence between them,
and every rule in it guards a silent failure: a staged plan beats the
reason-derived one because it was classified from the operations that actually
changed; the first frame beats both because there is nothing retained to reuse;
and the last two are **merged** rather than substituted, because a hover that
restyles and a commit that reconciles are both owed by this one frame.

It is a file of its own for the same reason [`LayoutClassifier`] is: refining a
plan against what the styles actually changed is an entity, so choosing the plan
in the first place is its sibling — the coarse half of one two-step
classification (§3.4 phase 5). Its interface is four scalars in and one
`FramePlan` out; its internals are the four rules above.

[`FrameReason`]: crate::frame::FrameReason
[`FramePlan`]: crate::runtime::FramePlan
[`LayoutClassifier`]: crate::runtime::LayoutClassifier
