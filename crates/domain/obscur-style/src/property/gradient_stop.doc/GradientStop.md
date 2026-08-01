One color stop of a CSS gradient, as authored: a [`StyleColor`] at an optional
normalized `position` along the gradient line (spec §5.2).

`position` is `0.0..=1.0` (0% = the gradient start, 100% = its end). It is
[`None`] when the stop was authored without an explicit position, meaning the
used value is auto-distributed between its positioned neighbours. The CSS
compiler resolves those to explicit positions at lower time, so a lowered
gradient's stops all carry [`Some`]; the [`None`] case is retained so a
partially-positioned authored gradient can be modelled without loss.

This is the style-side stop; it deliberately has **no dependency on the
scene**. The scene coordinator converts it to the renderer's
`obscur_scene::paint::GradientStop` when it builds the gradient brush.

Because [`StyleColor`] and `position` carry `f32` data, this type derives
[`PartialEq`] but not [`Eq`]/[`Hash`].
