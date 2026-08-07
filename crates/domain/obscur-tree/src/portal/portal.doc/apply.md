Enacts the portal on `tree`.

This sets the **source node's visual parent** to the target's host
([`VisualParent::of`]), **leaves its logical parent unchanged**
(preserving style/theme scope, spec §4.14), and flags the **host** node
with [`NodeFlags::PORTAL_HOST`].

Returns `true` when both the source and the host are live and the portal
was applied; returns `false` (a no-op) otherwise. It is defensive about
stale ids: if either endpoint does not resolve to a live node, nothing is
mutated and `false` is returned, so the call is panic-free.

# No production caller

This method has **zero** callers outside this file's own unit tests, and it is
half of a mechanism rather than the mechanism: it marks the two endpoints and
maintains no visual child list, so nothing downstream can enumerate what has
been portalled anywhere. Nor does anything read `visual_parent()`. See
[`Portal`] and `docs/api/12-reality.md`, row *Portals*.
