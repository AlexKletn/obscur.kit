Splits the display list at each
[`BackdropFilter`](PaintCommand::BackdropFilter) into an ordered list of
[`SceneSegment`]s the renderer composites in sequence (spec §5.2, CSS
`backdrop-filter`).

`backdrop-filter` frosts the frame **as accumulated so far**, so it is
order-dependent and cannot live inside one Vello scene. This walk breaks the
command stream at every backdrop boundary: the commands between boundaries
become a [`SceneSegment::Content`] (converted exactly as
[`Self::convert_with_blurs`], so each still extracts its own `filter: blur`
layers), and each boundary becomes a [`SceneSegment::Backdrop`] carrying the
frost region + radii + radius. The renderer renders and composites each
content segment onto an accumulating frame, then frosts that frame in place
at each backdrop boundary.

# Limitation

Each content run is converted as an **independent** scene, so a `PushClip` /
`PushTransform` left open across a backdrop boundary is closed at the run's
end and does not carry into the next run (its pops there are ignored). A
backdrop-filter nested under such an ancestor is therefore approximate; the
common top-level acrylic panel is exact. A list with no backdrop-filter
returns a single `Content` segment equal to `convert_with_blurs`.
