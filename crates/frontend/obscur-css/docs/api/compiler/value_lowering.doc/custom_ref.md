Mints a [`PropertyValue::CustomRef`] when this one component is a `var()`
reference, and [`None`] for anything else.

This is what lets a grammar that would otherwise reject a function — a border
colour, a box-shorthand side — carry the reference instead of failing, since
what the reference stands for does not exist until the cascade has run.
