Appends a [`PaintCommand::DrawText`] drawing the pre-shaped `run` at
`origin` (its top-left in absolute scene coordinates), painted with
`brush`. This is a leaf command; the run's glyphs were shaped once in the
`obscur-text` domain. Returns `&mut Self` for chaining.
