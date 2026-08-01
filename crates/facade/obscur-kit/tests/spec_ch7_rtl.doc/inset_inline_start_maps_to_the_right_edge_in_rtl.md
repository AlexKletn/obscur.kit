§7.3: "- logical CSS properties;" — the same mapping for
`inset-inline-start`.

TRIAGE OVERRULED 2026-07-19, and left failing. Triage held that the measured
`-20` is the correct answer and the expected `180` wrong. It is not. Two
steps compose here and only the second is right.

Step 1, the static position: a 100px block child in a 300px `direction: rtl`
content box sits flush against the line-right edge, at x = 200. This runtime
puts it at 0.

Step 2, the relative shift: in rtl `inset-inline-start` maps to `right`, and
`left` is auto, so by css-position-3.txt:792-795 ("If only one is auto, its
used value becomes the negation of the other, and the box is shifted by the
specified amount") the used `left` is -20 — the box moves 20px left of its
static position. That step is implemented correctly, which is what triage
observed.

200 - 20 = **180**, exactly the expected value. The measured -20 is
0 - 20: correct relative positioning applied to a wrong static position. The
case is a second symptom of the same registered defect as
`margin_inline_start_maps_to_the_right_edge_in_rtl`, not a bad test, so
nothing here is weakened.
