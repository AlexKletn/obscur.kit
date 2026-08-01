The live scroll state a scroll container is painted with (CSS Overflow 3
§scroll-container): its current vertical scroll position and the scrollable
content height that sizes the scrollbar thumb.

The runtime owns the authoritative per-node scroll position + content height
(see [`scroll`](super::scroll)); it hands a map of these to the coordinator so
composition stays a **pure** function of its inputs. A node **absent** from
that map is not a scroll container (or does not overflow), so no scroll
transform and no scrollbar are emitted — byte-for-byte identical to before
scroll containers existed (the regression guard).
