A caching decorator that memoizes [`shape`](TextShaper::shape) results for an
inner [`TextShaper`] (spec §5.10).

The runtime shapes each text node up to three times per frame — once to
**measure**, once to **arrange**, and once to **compose** — and re-shapes
every frame even when nothing changed. Wrapping the active shaper in a
`CachingShaper` collapses all of that to a **single** call to the inner
engine per unique request: the first `shape` for a given text/size/face
invokes the inner shaper and stores the [`GlyphRun`]; every later identical
request (this frame or a future one) returns a clone of the cached run. The
output is byte-identical to shaping without the cache — this is a pure
performance change.

[`can_shape`](TextShaper::can_shape) and [`line_metrics`](TextShaper::line_metrics)
delegate straight to the inner shaper (both are cheap; only shaping is
cached).

The cache is single-threaded interior state (`RefCell`): the runtime shapes
on one thread, and `Box<dyn TextShaper>` carries no `Send`/`Sync` bound.

# Bound

The cache holds at most [`MAX_ENTRIES`](Self::MAX_ENTRIES) runs. When a new
entry would exceed the cap the whole map is cleared before inserting — a
simple, deterministic, occasional flush that keeps memory bounded without an
eviction-order dependency (headless output never observes iteration order,
only the single returned value).
