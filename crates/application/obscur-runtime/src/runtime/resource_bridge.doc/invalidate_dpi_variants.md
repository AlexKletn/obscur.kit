Drops every resident decoded image so the next frame's resolve pass loads it
again against the current device scale's DPI-variant ladder (spec §5.16).

Called only when the scale change actually selects different asset files —
`requested_density` is what decides that, so a `2.5` → `3.0` nudge (both
`@3x`) costs nothing while a `1.0` → `2.0` display change reloads. The
resident URLs go through [`release_evicted`](Self::release_evicted) so their
hub bookkeeping is dropped too: a URL left in `image_handles` reads as
"already attempted" and would never load again.

A load already in flight at the old density is not cancelled — it does not
need to be. Releasing its handle makes the hub reject its completion as
stale, so its pixels never reach the cache.
