Creates a font-less shaper that shapes only when a request's face carries
[`FontSource::Memory`](obscur_text::font::FontSource::Memory) bytes.

Without any font, [`shape`](TextShaper::shape) returns an empty run and
[`can_shape`](TextShaper::can_shape) returns `false`. Pair it with
[`with_faces`](Self::with_faces) so the faces a
[`FontRegistry`](crate::FontRegistry) resolves also size their own line
boxes.
