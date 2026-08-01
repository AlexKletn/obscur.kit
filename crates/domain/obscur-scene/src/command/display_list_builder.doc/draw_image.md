Appends a [`PaintCommand::DrawImage`] drawing the decoded `image` as one
copy scaled to `tile`, clipped to `region`, and tiled along the `repeat_x` /
`repeat_y` axes to fill `region`. This is a leaf command (no nesting effect).
A full-box stretch is `tile == region` with both repeat flags `false`.
Returns `&mut Self` for chaining.
