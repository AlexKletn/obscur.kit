One of the four screen edges.

A dock is attached to exactly one of them, which is a narrower statement than
[`LayerAnchor`](super::LayerAnchor) makes: an anchor is a set, and most of its
combinations do not describe a dock at all. This type exists because some
platforms accept only the narrow statement, so
[`LayerAnchor::docked_edge`](super::LayerAnchor::docked_edge) has to have
something to return.
