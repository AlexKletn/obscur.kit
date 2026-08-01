Records the eviction of an entry that freed `freed_bytes`, decrementing
the live entry count and byte total (spec §5.19 eviction).

Uses saturating subtraction so the counts can never underflow.
