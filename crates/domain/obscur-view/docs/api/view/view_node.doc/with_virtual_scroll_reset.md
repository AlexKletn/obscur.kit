Tags a virtual-list marker with a **scroll-reset token** (spec §4.9): when the
token differs from the one the runtime last saw for this list, the list's
scroll offset is reset to the top before its window is computed.

Use it when the same list shows *different content* over time — a file manager
entering a new directory — so a scroll offset that belonged to the old content
is discarded rather than carried into the new. Pass a value that changes
exactly when the content is replaced (a generation counter is the usual shape);
a list whose content only grows or shrinks in place should not tag one, so its
position survives. No-op on a node that is not a virtual-list marker.
