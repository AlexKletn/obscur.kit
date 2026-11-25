Opens the dispatch phase and hands over the queued events, or [`None`] when
nothing is queued — in which case the previous frame's results are cleared
and the latch is not set.

Every non-[`None`] return **must** be closed with
[`end_dispatch`](Self::end_dispatch).
