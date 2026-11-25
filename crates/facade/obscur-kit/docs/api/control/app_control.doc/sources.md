The authored sheets as identified, versioned CSS sources (spec §4.12).

Position is identity: the first sheet is always `SourceId(1)`, so a sheet that
keeps its place keeps its cache entry across a replacement instead of piling up a
new one. `SourceId(0)` is reserved for anonymous sources — which are never
cached, which is what the authored set used to be.
