Enacts the portal on `tree`.

This sets the **source node's visual parent** to the target's host
([`VisualParent::of`]), **leaves its logical parent unchanged**
(preserving style/theme scope, spec §4.14), and flags the **host** node
with [`NodeFlags::PORTAL_HOST`].

Returns `true` when both the source and the host are live and the portal
was applied; returns `false` (a no-op) otherwise. It is defensive about
stale ids: if either endpoint does not resolve to a live node, nothing is
mutated and `false` is returned, so the call is panic-free.
