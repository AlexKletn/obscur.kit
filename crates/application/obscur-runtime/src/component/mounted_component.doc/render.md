Renders the component against the node's authored `props`, through its
error boundary (spec §1.7).

`signals` supplies the current generation of every observable source, so a
dependency recorded by [`ComponentContext::track`] captures the generation
it was actually taken at.

# A failed render never returns raw

There is no `Result` here on purpose: a caller handed an error would have
to decide what happens to the subtree, and that decision is the
boundary's. A failure comes back as [`GuardedRender::Failed`] with the
boundary's [`RecoveryResult`](crate::recovery::RecoveryResult) already taken — and with the dirty mark
already set to match it, because only a *retry* wants this component
rendered again with the same props. Leaving it stale after any other
decision would re-fire the failure, and its diagnostic, every frame.
