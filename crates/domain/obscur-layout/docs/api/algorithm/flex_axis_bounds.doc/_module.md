One flex item's min/max bounds on one axis, and the clamps that apply them.

Pure style arithmetic: every function here reads a [`LayoutStyle`] and a
container extent and returns a number. Nothing measures, so nothing needs the
child measurer — which is exactly the seam that separates this file from
[`super::flex_main_probe`], its opposite number. The main axis and the cross
axis get the same treatment twice ([`main_bounds`]/[`cross_bounds`],
[`clamp_floor`]/[`clamp_cross`]) because css-flexbox-1 states the rule twice:
§4.6 clamps the main size, §9.4 step 11 clamps the cross size.

Which of `width`/`height` is "main" depends on the container's
`flex-direction`, so every function takes `is_row` and answers for the axis
that direction makes main — the mapping the rest of the algorithm relies on
never having to repeat.
