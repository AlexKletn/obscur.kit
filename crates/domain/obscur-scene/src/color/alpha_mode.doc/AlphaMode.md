Whether a color's or image's alpha is premultiplied into its color channels
(spec §5.7).

Authored colors are [`AlphaMode::Straight`]: the color channels are
independent of the alpha channel. Compositing operates on
[`AlphaMode::Premultiplied`] data, where each color channel has already been
scaled by alpha, which makes source-over blending a simple linear
combination.
