Scrolls the scroll container `node` by `dy` logical px (positive = toward the
content's end), clamped to the scrollable range, and returns the resulting
`scroll_top`. A `node` that is not an overflowing scroll container is
ignored and `0` is returned (CSS Overflow 3 §scrolling).
