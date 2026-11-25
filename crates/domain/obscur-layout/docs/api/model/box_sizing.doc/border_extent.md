Maps an explicit authored size `size` into the element's **border** extent
along one axis, given the axis's total border + padding `bp`.

This is the inverse of [`content_extent`](Self::content_extent): it yields
the border-box extent an explicitly-sized element occupies. Under
[`ContentBox`](Self::ContentBox) the authored value is the content box, so
the border + padding is **added** (a `width: 0` element with a 10px
horizontal border occupies a 10px border box). Under
[`BorderBox`](Self::BorderBox) the authored value already *is* the border
box, so it passes through unchanged.

A parent formatting context uses this to size an in-flow child from its own
explicit `width` / `height` without collapsing away the child's border +
padding (which would otherwise invert a degenerate CSS border-triangle box).
