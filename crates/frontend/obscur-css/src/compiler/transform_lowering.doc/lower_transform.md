Lowers a CSS 2D `transform` value into a [`PropertyValue::Transform`] carrying
the composed affine (spec §5.2).

`none` (the initial) lowers to the identity transform. Otherwise the value is a
list of `<transform-function>`s — `translate`/`translateX`/`translateY`,
`scale`/`scaleX`/`scaleY`, `rotate`, `skew`/`skewX`/`skewY`, and
`matrix(a,b,c,d,e,f)` — each lowered to an affine and **composed left-to-right**
(`M = f1 · f2 · … · fn`, so the last-listed function is applied to a point
first). `translate` distances resolve to logical pixels: only absolute `px`
(and a unitless `0`) are honoured; a **percentage** translate is treated as `0`
because the box size is unknown at lower time (a documented approximation), and
any other unit makes the whole value unsupported. Returns [`None`] (degrading to
an unsupported feature) for an empty list, a non-function component, or an
unparseable function.
