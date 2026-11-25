An immutable, versioned snapshot of a [`RetainedTree`].

Heavy subsystems (layout, paint, accessibility) receive immutable, versioned
snapshots rather than borrowing the live tree (spec §1.6). A snapshot clones
the live node records into an owned map, captures the root, and stamps the
tree's [`VersionStamp`] at the moment of capture. It is **immutable after
construction**: it exposes only read accessors and no mutators.

A snapshot may be freely retained and later discarded as stale: because the
live tree advances its version on every structural mutation, a consumer can
compare [`version`](Self::version) against the live tree's current version to
decide whether its snapshot is out of date.
