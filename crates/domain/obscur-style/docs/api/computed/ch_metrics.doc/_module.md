The one input the computed-value step cannot compute for itself:
[`ChMetrics`].

Every relative unit CSS defines folds from numbers the resolver already holds
— `em` from the node's `font-size`, `rem` from the root's, `vw`/`vh` from the
viewport. `ch` is the exception, and the only one: CSS Values 4 §6.1.2 defines
it as *"the used advance measure of the `0` (ZERO, U+0030) glyph in the font
used to render it"*. That is a measurement, not an arithmetic — it needs a
font loaded and a glyph shaped, which is a capability no domain crate has or
should have.

So the resolver asks for it through this port rather than computing it. The
measurement is *threaded into* computed-value time instead of being deferred
to a consumer that happens to own a shaper — which is what made `ch` the last
unit to die at the layout↔paint seam: layout could measure and did, paint
could not and read `0`.

# Why the port takes a whole [`ComputedStyle`]

The `0` advance depends on the font actually selected — `font-family`, and the
`font-weight`/`font-style` that pick the face within it — as well as the size.
Rather than invent a font-description struct this crate would have to keep in
sync with the real one, the port takes the style that names them. The resolver
guarantees the style it passes already carries the properties an implementor
needs to select a face; see [`ChMetrics::zero_advance`].
