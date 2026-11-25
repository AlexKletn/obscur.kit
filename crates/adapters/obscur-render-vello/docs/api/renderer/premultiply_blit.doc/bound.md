The bind group of the last present, and the source view it was built from.

Both bindings this group holds outlive the frame: the source is the view of
`VelloRenderer`'s cached offscreen, which is rebuilt only when the surface
changes extent, and the sampler is a field of this blitter. The target is not in
the group at all — it is the render pass's attachment — so the one thing that
genuinely differs on every present is the one thing that cannot stale this.

The view is stored beside the group as the key, so the cache is *keyed*, not
*invalidated*: no site has to remember to clear it, and therefore no site can
forget to. `bind_source` carries the full list of events that can make the group
wrong and where each one is answered.
