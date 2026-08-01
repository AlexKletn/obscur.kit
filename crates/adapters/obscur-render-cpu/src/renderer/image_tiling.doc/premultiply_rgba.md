Premultiplies straight-alpha RGBA8 bytes into the premultiplied form
`tiny-skia` expects: each color channel is scaled by `alpha / 255` (rounded),
the alpha channel kept. The input length must be a multiple of four.
