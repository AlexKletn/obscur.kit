Converts an obscur-scene [`Stroke`] style into a [`kurbo::Stroke`].

Width, join, both caps, and the miter limit are carried across; the
stroke's brush is handled separately by the scene converter. A non-empty
[`Stroke::dash`] is mapped onto kurbo's dash pattern (widened to `f64`)
with its phase offset, so a dashed scene stroke rasterizes as dashes; an
empty dash leaves kurbo's default (solid) dash, byte-identical to before.
