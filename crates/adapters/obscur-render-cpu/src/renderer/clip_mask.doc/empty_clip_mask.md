The clip mask that admits nothing: a fresh [`tiny_skia::Mask`] is zero-filled,
and zero coverage draws nothing anywhere.

Used for a clip whose outline encloses no area, where "nothing passes" is the
answer rather than a backend limitation. Returns `None` only if the mask cannot
be allocated.
