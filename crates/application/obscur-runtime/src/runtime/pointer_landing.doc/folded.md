The event's position in the **unscrolled layout space** the arranged boxes
and the hit-test index are keyed by: [`screen`](Self::screen) with every
crossed scroll container's `scroll_top` added back.

Equal to [`screen`](Self::screen) in a frame with no scroll container, so a
document that never scrolls lands byte-identically to one built before
scroll containers existed.
