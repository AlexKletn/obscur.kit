Registers one face of a family together with the cut it actually is.

The form to reach for when an application ships more than one file for a
family: register the bold as a
[`FontFaceBytes`](crate::bootstrap::FontFaceBytes) carrying
[`FontWeight::BOLD`](crate::text::span::FontWeight::BOLD) and
`font-weight: 700` resolves to the real bold rather than to a regular face
thickened by the renderer.
