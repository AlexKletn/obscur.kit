Returns whether this diagnostic is stale relative to `current_version`
(spec §5.13 versioned diagnostics).

A diagnostic is stale when it was computed against an older document
version (`self.version < current_version`); a consumer discards stale
diagnostics because the flagged bytes may have since moved or changed.
