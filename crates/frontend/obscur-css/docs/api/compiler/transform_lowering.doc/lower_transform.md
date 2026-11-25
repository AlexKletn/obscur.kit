Lowers a CSS 2D `transform` value (spec §5.2).

`none` (the initial) lowers to the identity transform. Otherwise the value is a
list of `<transform-function>`s — `translate`/`translateX`/`translateY`,
`scale`/`scaleX`/`scaleY`, `rotate`, `skew`/`skewX`/`skewY`, and
`matrix(a,b,c,d,e,f)` — composed **left-to-right** (`M = f1 · f2 · … · fn`, so
the last-listed function is applied to a point first).

A list whose every distance is absolute composes here into a
[`PropertyValue::Transform`]. A list carrying a `translate` in a relative unit
(`2em`, `10vw`, `calc(1rem + 4px)`) CANNOT: the font size and the viewport are
computed-value-time facts. Rather than drop the declaration or fake a `0`, the
functions ride out as a [`PropertyValue::TransformFunctions`], which
`absolutize_value` folds against the node's length context.

A **percentage** distance is refused outright ([`None`] → an
`UnsupportedCssFeature`, so the facade raises a diagnostic): it resolves against
the element's own border box, which is a used value that no computed-value pass
can supply. `translate(-50%, -50%)` therefore reports itself instead of quietly
centring nothing — see `docs/api/12-reality.md`.

Returns [`None`] for an empty list, a non-function component, or an unparseable
function.
