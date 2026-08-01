Returns the paragraph orchestrator as a shared borrow.

This is the seam the layout phase measures text through: `SceneCoordinator`
owns the orchestrator, so the layout coordinator borrows it from here to
size text nodes with the **same** itemizer/resolver/shaper (and therefore
the same runs, advances, and line metrics) the scene phase later draws
with. Because `layout` and `scene_composer` are disjoint fields of the
runtime, this shared borrow coexists with a mutable borrow of the layout
coordinator.
