Sizing, justifying and cross-aligning **one** flex line.

The middle of the flex algorithm: css-flexbox-1 §9.7's flexible lengths are
resolved for this line ([`super::flex_flexible_lengths`]), each item's cross
size settled ([`super::flex_cross_size`]), the leftover main space handed to
the `auto` margins and then to `justify-content` (§8.1, in that order — the
ordering *is* the feature), and every item written into its box.

It is one entity despite the size of its argument list, and [`LineArgs`] is
the evidence rather than the exception: a flex line needs the whole container
context to be placed, and naming those inputs once, in one struct, is what
lets both the single-line and the multi-line caller reach this pass through
exactly one call shape. The two paths differ in one field — `stretch_target`
— and that difference is the whole of the no-wrap/wrap semantics.
