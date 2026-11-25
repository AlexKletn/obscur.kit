Returns the byte range that the inserted text occupies in the *post-edit*
text: `range.start .. range.start + inserted.len()`.

The splice replaces `range` (length `removed.len()`) with `inserted`, so
the inserted bytes begin where the removed bytes began (`range.start`) and
span `inserted.len()` bytes; text after the splice shifts by
`inserted.len() - removed.len()`.
