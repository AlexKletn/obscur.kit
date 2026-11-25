Push an affine transform onto the transform stack (CSS 2D `transform`,
spec §5.2). Subsequent drawing — up to the matching
[`PaintCommand::PopTransform`] — is mapped by the composition of every
transform on the stack, applied to absolute scene coordinates. The affine
is already resolved **about its transform origin** by the coordinator
(`translate(origin) · matrix · translate(-origin)`), so a renderer just
multiplies it into its current transform. This half of the pair opens the
nested region; a well-formed list balances it with a `PopTransform`.
