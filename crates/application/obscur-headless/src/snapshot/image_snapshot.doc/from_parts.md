Builds a **hash-only** snapshot from an already-computed content hash,
for callers that hashed the pixels themselves.

The result carries no raster, so [`compare_within`](Self::compare_within)
on it reports [`GoldenMismatch::PixelsUnavailable`] rather than a verdict.
