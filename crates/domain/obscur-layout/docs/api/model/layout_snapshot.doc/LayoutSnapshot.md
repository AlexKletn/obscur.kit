An immutable, versioned snapshot of a node's arranged layout, produced for
hand-off to paint and hit-testing (spec §1.6).

The snapshot pairs the arranged [`LayoutBox`] with the [`VersionStamp`] under
which it was produced, so downstream consumers can detect staleness without
re-reading mutable layout state.
