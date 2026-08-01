The [`VisibilityObserver`] — watches whether a node is visible (spec §4.4).

Like all geometry observers it reports the coalesced net change after layout
commit. A node is *visible* when its box has non-zero area and — if a
viewport is supplied — its box overlaps that viewport. The observer fires
only on a visible/hidden transition.
