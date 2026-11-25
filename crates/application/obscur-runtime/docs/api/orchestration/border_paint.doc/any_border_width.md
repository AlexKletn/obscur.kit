Whether any of the four widths is positive, i.e. whether the node has a border
to paint at all.

`false` is the common case (a node with no `border` authored), and it lets the
caller skip all three border shapes at once: with every width zero
[`uniform_border`] and [`uniform_dashed_border`] both return `None` and
[`border_fills`] returns an empty vector, so the skip is exactly equivalent to
running them.
