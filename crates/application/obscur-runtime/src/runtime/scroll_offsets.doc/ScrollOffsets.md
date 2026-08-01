The per-element vertical scroll positions (CSS Overflow 3 §scrolling) and the
scroll-chain-folded view of the layout boxes that is derived from them.

# Why these are one entity

The map is the source of truth and the projection is a **cache of it**: each
box in `folded` is its content-space box translated up by the summed offset of
every scroll container above it. So every operation that writes an offset must
also rebuild the projection, or the public geometry accessor reports a node at
the position it was painted at *before* the scroll. Written apart — a `HashMap`
on one struct, two projection fields on the same struct, and a `refresh_…` call
each writer had to remember — the failure is silent and only shows up in an
observer's coordinates.

Here the two cannot come apart: [`set`](Self::set) and [`shift`](Self::shift)
take the [`FoldInputs`] they need to refold, and do it in the same call. There
is no route to the map that writes an offset and leaves the projection behind,
except the one that names itself — [`seat`](Self::seat), whose doc says which
two callers may use it and why the frame refolds behind them.

The second half is on the **read** side. A stored offset is kept as written and
re-clamped on every read ([`clamped`](Self::clamped)), so a relayout that
shrinks a container's content never surfaces a stale over-scroll — and one that
grows it back restores the position the user had, which a clamp-on-write would
have thrown away.

# What it deliberately does not do

It holds neither the tree, the boxes nor the computed styles, so it cannot
resolve a [`ScrollContainer`] itself: the scrollable extent an offset is
clamped to is a fact about the **live layout**, and it arrives as an argument.
That is the same choice [`SurfaceGeometry`](super::SurfaceGeometry) makes about
the caches it must not drop, and it is what makes this type testable against a
hand-built container with no runtime in sight.
