Returns whether a reader at `self` can read a document written at
`other` (spec §2.5).

This is the semver-style rule: the majors must be equal, and `self`'s
minor must be at least `other`'s. A newer reader can read an older
document (additive minor changes are backward-compatible), but an older
reader cannot read a document that uses newer minor features, and no
reader can cross a major boundary.
