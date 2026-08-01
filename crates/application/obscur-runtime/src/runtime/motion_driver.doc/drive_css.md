Drives the installed CSS `transition`/`@keyframes` rules against the tree,
registering timelines with the [`MotionScheduler`] for any transitioned
property that changed value or any newly-matched animation (spec §4.17 S4).

`computed` is this frame's *resolved* style, sampled **before** the motion
overlay, so a transition's change-detection compares the settled cascade
target — not a mid-animation value — against the retained
[`transition_targets`](Self::transition_targets) baseline. The pass runs in
two stages to keep the borrow of `self.css_motion` disjoint from the
`&mut self` [`animate`](Self::animate) calls: it first *collects* the
actions to take (matching each rule's selector against every element node
via [`SelectorMatcher`], honouring recorded pseudo-state), then *applies*
them — updating the baselines and starting the timelines.
