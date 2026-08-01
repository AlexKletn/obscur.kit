Whether this entry's stored constraints exactly equal `constraints`.

Equality is the exact `f32` `PartialEq` comparison: a cache hit requires
identical constraints, so a measure under any different constraints
misses.
