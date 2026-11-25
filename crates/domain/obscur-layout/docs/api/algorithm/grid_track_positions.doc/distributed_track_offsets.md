Running start offsets for a track list with leftover free space distributed
per a content-distribution `mode` (`justify-content` on columns /
`align-content` on rows).

This generalises [`track_offsets`]: it computes the leftover
(`content_extent − (Σ non-collapsed sizes + gap * (n − 1))`) and, when it is
positive, shifts the first track by a `leading` offset and inserts an
`extra_gap` between adjacent tracks per [`content_distribution`]. The track
**sizes** are unchanged — only their **positions** move — so items placed in a
track move with it. With the [`FlexStart`](JustifyContent::FlexStart) default
(or no leftover) `leading` and `extra_gap` are both zero and this is
byte-identical to [`track_offsets`].
