Builds the clip mask in force after pushing `sk_path` onto a `parent` clip.

With no parent, the mask is the path's coverage over the whole buffer. With a
parent, the path is intersected into a clone of the parent's coverage. Returns
`None` only if a fresh [`tiny_skia::Mask`] cannot be allocated.
