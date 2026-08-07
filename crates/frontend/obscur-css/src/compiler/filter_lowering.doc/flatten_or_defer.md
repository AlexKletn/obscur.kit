Returns the list as a [`PropertyValue::Filter`] when every function is already
resolved, and as a [`PropertyValue::FilterFunctions`] when a `blur()` radius
still needs a length context.

The flatten is kept here — rather than always deferring to computed value — so
that the overwhelmingly common absolute list costs exactly what it did before.
