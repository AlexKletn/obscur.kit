Installs `shaper` as the inner single-run shaper, replacing the previous
one and leaving the itemizer and resolver in place.

The caller wraps the engine in whatever caching decorator it wants before
calling this; the orchestrator stores it verbatim. The paragraph cache is
flushed so no run from the old shaper survives.
