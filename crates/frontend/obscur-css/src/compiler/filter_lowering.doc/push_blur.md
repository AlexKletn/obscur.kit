Appends a `blur()` to the list: folded into a [`FilterOp::Blur`] when its radius
is already in pixels, dropped entirely when that radius is `0`, and kept as a
pending [`FilterFunction::Blur`] when it is not yet resolvable.
