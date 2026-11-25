One shape of a [`SceneVector`](crate::paint::SceneVector): an outline with an optional fill and an
optional stroke.

Both are optional and independent, exactly as in SVG: a shape may be filled,
stroked, both (two paints over one outline), or — for a `<path>` used only as
a clip source — neither.
