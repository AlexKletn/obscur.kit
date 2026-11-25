[`clamp_content`], with the intrinsic keywords already **measured** by the
caller (spec §4.2, CSS Sizing 3 §4.1).

Each bound is paired with the content-axis length it measured to, which is
[`Some`] only when that bound is an intrinsic keyword — the one case
[`Dimension::resolve`] cannot answer, because the answer lives in the content.
A caller that hands `(bound, None)` for a non-intrinsic bound loses nothing:
the length is resolved from the bound itself, exactly as in [`clamp_content`].
