Appends a [`PaintCommand::DrawShadow`] painting a CSS `box-shadow` for the
border box `rect` with corner `radii`, the resolved `color`, blur radius
`blur`, `spread`, `offset`, and the `inset` flag. This is a leaf command
(no nesting effect). Returns `&mut Self` for chaining.
