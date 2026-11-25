Stores `bytes` under `key`, tagged with `version` (spec §5.20 versioned).

Any existing entry for `key` is replaced, so a re-derivation at a newer
version supersedes the stale one.
