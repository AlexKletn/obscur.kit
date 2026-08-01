The full cache key for [`ParagraphShaper::shape_paragraph`]: everything that
can change the itemize → resolve → shape output.

[`ParagraphShaper::shape_paragraph`](crate::shaping::ParagraphShaper::shape_paragraph)
is width-independent — wrapping to a concrete width happens later in
`ParagraphLayout` — so its result depends only on the six inputs captured
here. Memoizing on this key lets the measure pass (wraps to the constraint
width) and the compose pass (wraps to the box width) share one shaped result
for an unchanged text node, across both phases and across frames.

The `f32` font size is stored via [`f32::to_bits`] so the key can derive
`Eq` and `Hash` (a raw `f32` cannot). The `families` fallback chain is
captured as its ordered list of [`FontFamily`] (each cheap and `Hash`), the
`attributes` directly (they are `Copy + Hash`), and the `default_face` by its
shaping-relevant identity — family, attributes, and the cheap
[`FaceSourceKey`] (never hashing font bytes). Two calls with the same key
produce byte-identical runs; any observable difference lands in a different
bucket.
