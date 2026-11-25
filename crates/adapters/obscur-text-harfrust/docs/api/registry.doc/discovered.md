Faces pulled in from the host on demand for characters nothing
registered covers — see [`with_system_fallback`](Self::with_system_fallback).

Kept apart from `fonts` so the registered chain's ids, order and search
stay exactly what they were: a discovered face is only ever consulted
after every authored family has already declined the run.

**Shared across clones.** The registry is cloned to be handed to the
runtime, and a face discovered on the runtime's copy must be visible to
the copy a host drains for the renderer — otherwise the glyph resolves
but nothing rasterizes it.
