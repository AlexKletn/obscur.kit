A [`ChildMeasurer`] backed by a fixed natural size per child index.

[`ChildMeasurer`]: super::ChildMeasurer

This exists so layout algorithms can be exercised in isolation — without a
runtime, a retained tree, or a text shaper. Each child index is
assigned one *natural* size; a measurement returns that natural size clamped
into the requested constraints (via [`BoxConstraints::constrain`]). An index
outside the backing vector measures as [`Size::ZERO`], so the measurer never
panics regardless of the indices an algorithm asks for.

It is deterministic and side-effect-free: repeated calls with the same
`(index, constraints)` always return the same size.
