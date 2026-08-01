Emits one [`BoxShadow`] as a scene
[`DrawShadow`](obscur_scene::command::PaintCommand::DrawShadow) over the border
box `bounds` with corner `radii` (spec §5.2).

The shadow's lengths are read as logical pixels (see [`border_width_of`](super::border_paint::border_width_of) for
why reading `px` here is sufficient, and for the two terms it is not); its
color falls back to `current` (the node's computed `color`, i.e. CSS
`currentColor`) when the author omitted it. The renderer applies the offset /
spread / blur when it rasterizes.
