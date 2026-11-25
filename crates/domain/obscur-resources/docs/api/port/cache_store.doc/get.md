Returns the bytes stored under `key`, but **only if** the stored entry's
version equals `version` (spec §5.20 compatibility-awareness).

A stored entry whose version differs from `version` is stale — derived
under an incompatible input or format revision — so it yields `None`
rather than incompatible bytes. `None` is also returned when no entry is
stored under `key` at all.
