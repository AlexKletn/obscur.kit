Lowers a `background-image` value into a gradient or `url(...)` property value.

The `linear-gradient(...)`, `radial-gradient(...)`, and `conic-gradient(...)`
image forms are modelled (into [`PropertyValue::LinearGradient`],
[`PropertyValue::RadialGradient`], and [`PropertyValue::ConicGradient`]
respectively); `repeating-linear-gradient(...)` lowers to the same
[`PropertyValue::LinearGradient`] carrying the repeating flag and its tiling
period. A `url(...)` image lowers to a
[`PropertyValue::BackgroundImageUrl`] carrying the **raw** authored URL/path
string (quotes stripped for `url("…")`, or the reconstructed body for the
unquoted `url(…)` form). The URL is *not* resolved here — the runtime hands it
to a resource loader/decoder.

**Multiple layers**: `background-image` accepts a comma-separated list of image
layers, in author order (first = top). This splits `values` on **top-level**
commas only — the argument commas inside each `linear-gradient(…, …)` /
`rgb(…, …)` live inside that component's [`ParsedValue::Function`] payload and
are never seen here, so they never split a layer. Each segment lowers with the
per-kind lowering. A single layer keeps the flat one-value result
([`PropertyValue::LinearGradient`] / …), **byte-identical** to before; two or
more layers yield [`PropertyValue::BackgroundLayers`]. `none` and any
unparseable / empty layer yield [`None`] (degrading to an unsupported-feature
diagnostic).
