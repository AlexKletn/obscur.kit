Resolves a text node's `letter-spacing` into extra per-cluster tracking, in
logical pixels, from its computed `letter-spacing` (CSS Text 3 §8.1).

Reads [`PropertyId::LetterSpacing`]: a [`PropertyValue::Length`] resolves to
its pixel value (which may be negative) — `px` verbatim, `em` × the node's
`font_size`, `rem` × `root_font_size` (CSS Values 4 §5.1.1); the initial
`normal` keyword, an absent property, a percentage (letter-spacing does **not**
accept `%` per CSS Text 3 §8.1 — dropped), or any other value resolves to
`0.0`, so an unauthored text node adds no tracking and shapes byte-for-byte as
before.

The same value feeds **both** the layout measure/arrange pass and the scene
compose pass (through the shared [`apply_spacing`](obscur_text::shaping::apply_spacing)
seam) so the measured box width and the painted advances agree.
