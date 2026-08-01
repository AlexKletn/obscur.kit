Converts a domain [`Path`] into a [`tiny_skia::Path`], or `None` when the
outline is empty or degenerate (`tiny_skia::PathBuilder::finish` rejects it).
