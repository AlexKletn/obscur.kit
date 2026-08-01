Returns a copy of this context with the measured `0`-glyph advance set to
`width`, in logical pixels.

Only a caller that can actually shape text can answer this, so it is a
builder rather than a [`new`](Self::new) parameter: a context built
without a shaper in reach stays honest about not knowing.
