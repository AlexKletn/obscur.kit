Computes the inclusive tile index range `[start, end]` covering the span
`[r0, r1]` with copies at `t0 + k * ts` (`ts > 0`), matching the CPU backend. A
non-repeating axis is the single index `0`; the range is padded by one on each
side and clamped to [`MAX_TILES_PER_AXIS`] copies (the `region` clip trims any
overdraw).
