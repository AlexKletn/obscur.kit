Returns a readout of `hovered` — its identity (CSS `type#id.class`),
component id, and absolute box. `None` when nothing is hovered or the
hovered node has no retained box yet.

An associated function: the readout is a pure projection of the frame and
does not consult the overlay flag, so a caller can ask for it whether or
not inspect mode is on — which is what lets a host print the readout beside
its own highlight.
