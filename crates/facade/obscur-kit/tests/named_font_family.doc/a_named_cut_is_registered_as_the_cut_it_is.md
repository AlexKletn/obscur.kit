Two files of one family register as two cuts of it, so `font-weight: 700`
resolves to the real bold rather than to a regular face the renderer thickens.

This is why an application face is a
[`FontFaceBytes`](obscur_kit::bootstrap::FontFaceBytes) and not just a name
and some bytes.
