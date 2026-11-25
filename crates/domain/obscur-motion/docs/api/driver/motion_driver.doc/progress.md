Returns the normalized progress this driver currently dictates (spec
§4.17).

The value is normally clamped to `[0, 1]` — each implementation documents
exactly how it derives and bounds it — and is what feeds a track's
`sample(t)`. Pure and deterministic: reading the same driver in the same
state always yields the same progress.
