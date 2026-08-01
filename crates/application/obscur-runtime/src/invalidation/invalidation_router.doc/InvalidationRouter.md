Routes classified invalidations to the domain accumulators and tracks which
pipeline phases are dirty for the current frame (§4.3).

The router does **not** walk the tree. It records node-level invalidations in
the [`StyleInvalidation`] and [`LayoutInvalidation`] accumulators and flags
which phases the frame pipeline must run.

# What its callers actually use

`FramePlan::for_frame` / `for_transaction` read the dirty-phase **booleans**
only (`needs_style()`/`needs_layout()`/`needs_scene()`) and drop the
accumulators unread. For `for_frame` that costs nothing — it routes only
*global* invalidations (node `None`), which record no per-node entry. For
`for_transaction` it is a real discard: `UiOperation::invalidation` names a
node for everything but `ReplaceRoot`, so a per-node set is built there and
thrown away. That is harmless today only because of what production actually
emits — `drag_coordinator.rs:257` sends `Rerender`/`ReplaceRoot`, which *are*
structural and relayout in full. It is **not** harmless in general: an
`InvalidateLayout` commit is neither (`frame_plan.rs:313` asserts
`!plan.reconcile()` for one), and `layout_phase`'s own router sees no computed-
style delta for it — `LayoutChanged` means geometry moved *without* the style
changing — so such a commit is silently skipped rather than relaid out. Nothing
constructs one outside tests, which is why no frame is wrong; the gap predates
the per-node set and is not closed by it.

`UiRuntime::layout_phase` uses both halves: it routes this frame's real
per-node style deltas through here, reads `needs_layout()` to decide **whether**
the layout phase runs, and hands the [`LayoutInvalidation`] this filled to
`LayoutCoordinator::layout_invalidated` to decide **which** contexts it
re-measures. So the spec's "recalculation starts at the nearest affected
context" is realised through this router for layout's measure step.

The [`StyleInvalidation`] this fills is still dropped unread by every caller:
the style phase's per-node restyle takes its set from `RestyleScope` reading the
live interaction state, on a path that does not involve this router at all.

The dirty-phase flags accumulate across calls to [`route`](Self::route) within
a frame and are cleared by [`reset`](Self::reset); the domain accumulators are
owned and reset elsewhere.
