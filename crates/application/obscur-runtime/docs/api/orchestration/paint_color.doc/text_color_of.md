Reads a node's computed text `color` as a scene [`Color`], defaulting to
[`Color::WHITE`] when absent.

Mirrors [`background_color_of`](super::background_paint::background_color_of) but reads [`PropertyId::Color`]. When the node
has no computed style, no `color` property, or a non-color value, the run
falls back to opaque white (visible on the example's colored bars).
