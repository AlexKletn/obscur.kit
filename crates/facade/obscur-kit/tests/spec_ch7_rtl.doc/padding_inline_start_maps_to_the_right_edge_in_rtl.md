§7.3: "- logical CSS properties;" — the same mapping for
`padding-inline-start`.

EXPECTED VALUE REPAIRED 2026-07-19: this asserted `180` in rtl, copied from
the sibling margin case, and 180 is arithmetically impossible here. The
padding is declared on the CONTAINER, which is `box-sizing: content-box;
width: 300px` — padding sits OUTSIDE that 300px content box and does not
shrink it. In rtl, `padding-inline-start` maps to `padding-right`
(css-writing-modes-4.txt:1510-1513), so the content box is still x ∈ [0, 300]
and a 100px child flush against its line-right edge has its left edge at
**200**, not 180. The ltr half (20) was and is correct: `padding-left: 20px`
starts the content box at 20.

The case is expected to keep failing, but now for one honest reason instead
of two: measured 0, i.e. ltr placement. That is the same defect the register
already carries as `margin_inline_start_maps_to_the_right_edge_in_rtl` —
block-level over-constrained resolution ignores `direction` — and this case
is a second symptom of it, not a second gap.
