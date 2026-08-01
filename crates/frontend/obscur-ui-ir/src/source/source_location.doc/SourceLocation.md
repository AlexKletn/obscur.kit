A single position within a source file (spec §2.5).

Records a byte `offset` together with the human-facing `line` and `column`
it corresponds to. Lines and columns are 1-based (the first character is at
line 1, column 1); the offset is 0-based. Ordering is by `offset` alone,
which is the canonical position within the source.
