Absolutizes the one RESOLVED filter op that carries lengths (`drop-shadow(...)`).

A `blur(<length>)` in this list is already an `f32` — the CSS lowering flattens a
`px` radius directly. A radius it could NOT flatten (`blur(1rem)`) never reaches
this function: it arrives as a `PropertyValue::FilterFunctions`, which
`absolutize_filter_functions` resolves first.
