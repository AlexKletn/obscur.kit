Computes one line box's used `(height, baseline)` from this CSS 2.1
§10.8.1 strut and the fonts on the line.

The line's ascent `A = max(strut.ascent, max seg ascent)` and descent
`D = max(strut.descent, max seg descent)` take the strut (the element's own
primary font) as an always-present floor — an empty line still gets the
strut's own box. With content box `A + D` and used `line-height`
`L = strut.line_height`: when `L > A + D` the excess splits as half-leading
`(L − (A+D))/2` above and below, giving `height = L`,
`baseline = A + half_leading`; otherwise `height = A + D`, `baseline = A`.
The `max` is order-independent, so the result is deterministic.
