Maps an explicit authored size `size` into the element's **content**
extent along one axis, given the axis's total border + padding `bp`.

Under [`ContentBox`](Self::ContentBox) the authored value already *is* the
content extent, so it passes through unchanged. Under
[`BorderBox`](Self::BorderBox) the authored value is the border box, so the
border + padding is subtracted and the result floored at `0` (a box
smaller than its own border + padding collapses its content to zero rather
than going negative).
