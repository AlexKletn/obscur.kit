Returns `font`'s vertical [`LineMetrics`] scaled to `font_size`, or `None`
when [`get`](Self::get) has no metrics for it (or they carry a zero
units-per-em, which cannot be scaled).

Design units become logical pixels through `font_size / units_per_em`. The
font's negative-below-baseline descent is turned into the positive
magnitude [`LineMetrics`] stores.
