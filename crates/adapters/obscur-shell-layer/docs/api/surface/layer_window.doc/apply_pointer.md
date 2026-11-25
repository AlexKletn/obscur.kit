Make the surface clickable, or let the pointer through it.

Wayland states this as an **input region** on the plain `wl_surface`, not as
anything the layer shell says: an empty region means no pointer or touch event
ever lands on the surface, and the compositor tries whatever is behind it
instead. A null region — the initial value — means the whole surface takes
input.

The region is double-buffered like the rest of the surface's state, so this
takes effect at the caller's next commit, which is the commit
[`apply`](fn.apply.html) is part of.
