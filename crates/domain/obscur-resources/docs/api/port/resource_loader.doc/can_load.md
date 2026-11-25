Returns whether this loader can resolve `source` (spec §5.15).

A caller consults this to route a source to a loader that understands it
before calling [`Self::load`].
