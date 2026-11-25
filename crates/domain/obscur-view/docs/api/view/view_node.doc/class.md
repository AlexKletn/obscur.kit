Sets this element's `class` attribute (spec §4.13), the input the cascade
matches `.name` rules against. A no-op on a text node, like
[`with_prop`](Self::with_prop).

The shorthand for the single most common styling operation — it is exactly
`self.with_prop(ViewProperty::new(CLASS, ViewValue::Text(class.into())))`.
Like a `class="a b"` attribute, `class` carries the whole space-separated
list; call it once with every class rather than chaining it.
