Below this magnitude a scale factor is treated as degenerate (a collapsed
axis), so normalization substitutes a safe unit basis instead of dividing by
~zero. Kept generous relative to `f32::EPSILON` so ordinary sub-pixel scales
still decompose exactly.
