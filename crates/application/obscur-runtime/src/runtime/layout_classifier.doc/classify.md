Refines the plan's conservative layout flag into **what** this frame's
layout phase must do, from the actual per-node changes (spec §3.4 phase 5
"classify invalidation" + step 8 "perform incremental layout", §4.3).

[`FramePlan`] derives its layout flag from the frame's [`FrameReason`]
alone, so *any* hover asks for a full relayout even when the rule only
shifts a fill. Layout's real inputs are the tree structure, the viewport +
sticky scroll offset, and the **layout-affecting** computed properties
([`PropertyId::affects_layout`]). For the two hot non-structural reasons
this routes the real per-node style deltas through the
[`InvalidationRouter`] — feeding the domain accumulators the invalidation
model was built for — and returns:

* [`Skip`](LayoutPhase::Skip) when no node's box can have moved, so the
  whole phase is skipped and the retained boxes (and hit-test index) carry
  over;
* [`Invalidated`](LayoutPhase::Invalidated) carrying the very
  [`LayoutInvalidation`] the router filled, naming the contexts that must be
  re-measured — everything else is served from the retained
  [`MeasureCache`];
* [`Full`](LayoutPhase::Full) when the frame cannot say, which re-measures
  the tree exactly as it always did.

Conservative by construction — it only ever *removes* provably redundant
work:
* a plan that already skips layout is left alone;
* a first frame, or a reconcile that ACTUALLY changed the tree, lays out in
  full;
* any reason outside the refined set below lays out in full;
* a moved viewport / scroll offset, or a changed node count, lays out in
  full;
* an unknown property is layout-affecting by default.

# Why `reconciled` and not `plan.reconcile()`

The reactive path ([`UiRuntime::run_due_frame_reactive`](super::UiRuntime::run_due_frame_reactive))
classifies EVERY frame as a structural [`FrameReason::UiCommit`], because the
builder may have produced a different tree and that must be reconciled. So a
windowed host — which drives every hover through the reactive path — would
hit `plan.reconcile() == true` on every frame and never reach this
refinement at all. But reconcile itself reports whether it changed anything:
it diffs text content, attributes and event bindings *by value*, so an empty
commit proves the tree is byte-identical. Gating on that actual result rather
than the plan's conservative upper bound is what makes a reactive hover
layout-free.

# Why the reason allowlist stays

The checks below cover layout's inputs for the listed reasons only. Notably
[`ResourceCompletion`](FrameReason::ResourceCompletion) is NOT refined: a
font or image that finished loading changes text metrics / intrinsic sizes,
and neither is a computed style nor a tracked layout input, so its deltas are
invisible here and it must keep laying out.
# Which nodes it has to ask about

That depends on how the style phase produced `computed`, which is what
`delta` carries. A [`Full`](StyleDelta::Full) pass re-resolved everything, so
every node is a candidate and the previous styles are still in
the retained styles. An [`Incremental`](StyleDelta::Incremental) pass re-resolved
only the invalidated nodes, so those are the only candidates — every other
node's style was not recomputed and therefore cannot have changed — and it
hands over exactly their previous styles. A hover on one node then costs one
property comparison here, not one per node in the tree.
