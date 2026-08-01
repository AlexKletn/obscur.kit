Resolves one length-valued property into a [`Dimension`], defaulting to
[`Dimension::Auto`] when the property is absent or cannot be resolved.

Absolute pixels and bare percentages pass through directly (percentages stay
symbolic for layout to resolve against the containing block); relative and
viewport units and `calc()` expressions are folded to pixels against `ctx`.
