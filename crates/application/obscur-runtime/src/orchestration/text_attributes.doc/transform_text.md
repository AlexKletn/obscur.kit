Applies `transform` to `content` **before** shaping (CSS Text 3 §2.1),
returning the cased string.

This is the pre-shape seam: the transformed string flows through itemization,
shaping, and the paragraph cache key, so a transformed node is a distinct
cache entry and the measure and compose passes — which both run `content`
through this same function — shape identical text.

[`TextTransform::None`] borrows `content` unchanged (zero-copy,
byte-identical). `uppercase`/`lowercase` use Rust's Unicode-aware
[`str::to_uppercase`]/[`str::to_lowercase`]. `capitalize` uppercases the first
**cased letter** of each whitespace-delimited word (the first letter, not a
leading symbol or digit), leaving the rest untouched. Locale-tailored casing
(e.g. Turkish dotless *i*) and the full grapheme/word-boundary subtleties are
documented refinements over this common Unicode path.
