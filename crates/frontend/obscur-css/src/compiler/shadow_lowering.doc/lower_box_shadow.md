Lowers a `box-shadow` value into a [`PropertyValue::BoxShadow`] list (spec
§5.2).

`box-shadow: none` lowers to an **empty** list (painting nothing). Otherwise
the value is a comma-separated list of `<shadow>`s, each lowered by
[`lower_one_shadow`]; the list order is preserved (CSS paints the first-listed
shadow on top). Returns [`None`] (degrading to an unsupported-feature
diagnostic) for an empty layer, an unparseable shadow, or no shadows at all.
