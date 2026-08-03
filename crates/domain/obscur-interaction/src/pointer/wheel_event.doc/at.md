Returns the event aimed at a different point.

The runtime uses it to hand a handler the FOLDED position — the point in the
space the node was laid out in — rather than the raw host coordinate it
received, so a listener and a hit test agree about where the notch landed.
