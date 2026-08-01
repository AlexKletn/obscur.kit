A scroll-driven [`MotionDriver`]: a scroll offset mapped onto `[0, 1]` across
a scroll range (spec §4.17 "scroll signals — scroll-linked animations").

The driver maps the current scroll `offset` from the range `[start, end]`
that the animation spans onto normalized progress via
`(offset - start) / (end - start)`, clamped to `[0, 1]`. This is the **domain
half** of a scroll-linked animation: the driver does no scrolling and reads
no viewport — the interaction/scroll subsystem feeds it the latest offset via
[`set_offset`](Self::set_offset), and the driver deterministically turns that
into progress. A degenerate range (`start == end`) has no width to normalize
against and yields progress `0`.
