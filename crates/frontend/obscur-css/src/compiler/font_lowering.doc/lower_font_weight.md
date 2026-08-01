Lowers a `font-weight` value on the CSS `1..=1000` weight axis.

A bare number is taken verbatim, and the absolute named keywords map to their
numeric equivalents (`normal` → 400, `bold` → 700), both as a
[`PropertyValue::Number`]. The *relative* keywords (`bolder` / `lighter`)
lower to the reserved [`layout_keyword`] handles
[`FONT_WEIGHT_BOLDER`](layout_keyword::FONT_WEIGHT_BOLDER) /
[`FONT_WEIGHT_LIGHTER`](layout_keyword::FONT_WEIGHT_LIGHTER) instead of fixed
numbers, so the computed-value step can resolve them *relative* to the
parent's computed weight (§4.14). The CSS-wide keywords (`inherit` /
`initial` / `unset`) still take priority. Any other value yields [`None`] so
the declaration degrades to an unsupported-feature diagnostic.
