Returns the inverse edit: applying it to the post-edit text restores the
pre-edit state.

The inverse replaces the [`inserted_range`](Self::inserted_range) — where
the inserted text now sits, `range.start .. range.start + inserted.len()`
— putting `inserted` and `removed` back in swapped roles, and swaps the
before/after selections. Inverting twice is the identity for a real edit.
