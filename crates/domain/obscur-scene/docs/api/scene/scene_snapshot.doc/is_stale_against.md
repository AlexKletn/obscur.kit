Whether this snapshot is stale relative to a `current` scene generation.

Returns `true` when `current` is strictly newer than this snapshot's
generation, i.e. the live scene advanced its content after the snapshot
was taken, so the snapshot may be discarded (spec §1.6).
