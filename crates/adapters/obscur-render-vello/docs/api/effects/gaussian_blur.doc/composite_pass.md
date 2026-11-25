Records the premultiplied-over composite of `input` onto `target` at the
target-space NDC `rect` (`[x0, y_top, x1, y_bottom]`).

`mask` optionally clips the composite to a rounded-rect panel shape (the
backdrop-filter frost); `None` composites the whole quad (the `filter: blur`
path), byte-identical to the pre-mask uniform (coverage is a constant `1`).
