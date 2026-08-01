Resolves a reserved `JUSTIFY_*` keyword symbol to a [`JustifyContent`],
defaulting to [`JustifyContent::FlexStart`] for an absent or unrecognized
value. Shared by `justify-content` (flex main axis / grid columns) and
`align-content` (grid rows), which use the same closed keyword set.
