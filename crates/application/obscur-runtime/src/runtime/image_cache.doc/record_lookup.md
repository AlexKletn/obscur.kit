Reports whether `url` is resident, recording the lookup as a hit or a miss
and stamping a hit's recency (spec §5.19 metrics).

This is the recency signal [`EvictionPolicy::Lru`] and the
[`VisibilityAware`](EvictionPolicy::VisibilityAware) tiebreak order by, so
every frame's resolve pass must ask through here rather than reading
[`images`](Self::images) directly.
