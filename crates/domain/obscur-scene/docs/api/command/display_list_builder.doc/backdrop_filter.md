Appends a [`PaintCommand::BackdropFilter`] frosting (blurring) the backdrop
— everything already painted behind this point — within the border box
`bounds`, clipped to the rounded `radii`, by the CSS blur radius `blur`.
This is a leaf command (no nesting effect); it must be issued **before** the
element's own background so the element paints over the frosted backdrop.
Returns `&mut Self` for chaining.
