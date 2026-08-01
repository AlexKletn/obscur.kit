The [`ResizeObserver`] — watches a node's size only (spec §4.4).

Like all geometry observers it reports the coalesced net change after layout
commit. Unlike a [`LayoutObserver`](super::LayoutObserver), it watches only
the [`Size`]: a position-only move (same size, different origin) does not
fire.
