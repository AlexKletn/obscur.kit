One flex item's cross-axis size (css-flexbox-1 §9.4).

Three sources answer the same question and the caller must not care which:
an explicit cross size on the item, the stretch target it fills when
`align-self: stretch` applies, or a measurement of the item at its already
flexed main size. All three then pass through the same §9.4 step 11 clamp, and
clamping *here* rather than at the three call sites is what keeps an item's
hypothetical cross size and its used cross size consistent.

The long doc below is the load-bearing part: the measure fixes the item's main
axis to its used size, which is the right offer but the wrong containing
block, and stating the percentage basis is what keeps those two apart.
