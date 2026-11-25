A typed classification of *why* a layout context must be recomputed
(spec §4.3).

§4.3 requires that changes be classified into typed reasons rather than
tracked through scattered ad-hoc dirty flags. Each variant identifies the
kind of change that reached a context.

# The variant itself is still write-only

The [`LayoutInvalidation`](super::LayoutInvalidation) a frame fills **is** read
— its node set drives the incremental measure pass (see
`LayoutCoordinator::layout_invalidated`) — but that pass branches on *which
nodes* are invalidated, never on *why*: it treats every reason below
identically and conservatively. So these variants are, today, a classification
the runtime records and no consumer discriminates on.

Reading them is what would let propagation follow §4.3's finer rules (upward
only on geometry change, downward only on constraint change) rather than the
whole-ancestor-chain closure the measure pass currently takes; that needs the
per-context [`LayoutDependency`](crate::context::LayoutDependency) wiring
nothing populates yet.
