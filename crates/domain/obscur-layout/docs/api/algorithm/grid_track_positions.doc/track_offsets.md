Running start offsets for a track list: `offsets[i] = origin + Σ_{k<i}
(sizes[k] + gap)`. Length is `sizes.len()`.

A `collapsed[i]` track (an empty `auto-fit` track) contributes neither its size
nor a following gap, so the gaps on either side of it merge into one and the
occupied tracks pack together (CSS §4.1). With no collapsed track this is
byte-identical to the plain running-offset accumulation.
