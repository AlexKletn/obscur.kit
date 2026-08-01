Computes the inclusive tile index range `[start, end]` covering the span
`[r0, r1]` with copies at `t0 + k * ts` (`ts > 0`). A non-repeating axis
(`repeat == false`) is the single index `0`. The range is padded by one on each
side (so a partially-visible edge tile is drawn) and clamped to
[`MAX_TILES_PER_AXIS`] copies — the `region` clip trims any overdraw.
