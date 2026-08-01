The `z-index: auto` keyword (the default; the element does not establish its
own stacking level and stacks with its parent). The `<integer>` form of
`z-index` lowers to a [`PropertyValue::Number`](crate::property::PropertyValue::Number)
instead, so this reserved handle carries only the `auto` case across crates.
