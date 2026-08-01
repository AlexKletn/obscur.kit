The `transform` property (a resolved `Transform` affine; NOT inherited).
`transform: none` lowers to an absent value. The scene coordinator
brackets the element's whole subtree paint with the composed affine,
resolved about [`TransformOrigin`](Self::TransformOrigin).
