Lowers a `background-position` value into a [`PropertyValue::BackgroundPosition`]
(spec §5.2, CSS Backgrounds §3.6).

Accepts one or two components from
`[ left | center | right | top | bottom | <length-percentage> ]`. The edge
keywords lower to percentages (`left`/`top` = 0%, `center` = 50%,
`right`/`bottom` = 100%); axis keywords are assigned to their axis regardless
of order (`bottom right` == `right bottom`), and lengths / `center` fill the
remaining axes in component order. A single component leaves the other axis
centered. Any other syntax yields [`None`].
