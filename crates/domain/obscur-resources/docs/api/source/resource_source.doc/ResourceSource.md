Where a resource's bytes come from (spec §5.15 sources).

This is a pure *description* of a resource's origin that a loader port resolves
later; it performs no I/O itself. Per spec §5.15 the recognised source kinds
are: embedded (baked at build), file, memory, URL (through a user/host loader),
generated content, plus a host document handle for picked files (spec §5.16).

The bytes-in-hand kinds ([`EmbeddedSource`], [`MemorySource`]) can be resolved
entirely within the kit; the remaining kinds — file, URL, and document — are
resolved outside the kit by a loader or host. In particular the kit performs no
arbitrary network access by itself (spec §5.15): a [`ResourceSource::Url`] is
only a request that a user/host loader chooses to fulfil.
