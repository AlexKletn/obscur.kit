Component-level value lowering: the grammar every longhand shares (§4.14).

One parsed component becomes one [`PropertyValue`] — a dimension, a
percentage, a bare number, a keyword, a `var()` / `env()` reference or a math
function. Property-aware lowering (`lower_for_property`) reaches this path
only after it has failed to recognise the property's own grammar, so this is
the crate's floor: what a declaration means when nothing more specific
applies. Every specialised lowerer in this folder is written against the same
unit vocabulary, which is why `dimension_length`, `angle_to_degrees` and
`css_wide_keyword` live here rather than beside any one of them.
