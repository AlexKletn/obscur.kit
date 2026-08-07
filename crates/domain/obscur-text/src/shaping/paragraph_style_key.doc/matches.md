Whether this key describes the same style as the borrowed shaping inputs.

Allocation-free by construction: every field is compared against what the
caller already holds. This is the whole reason the style half is a separate
key — see [`ParagraphStyleKey`].
