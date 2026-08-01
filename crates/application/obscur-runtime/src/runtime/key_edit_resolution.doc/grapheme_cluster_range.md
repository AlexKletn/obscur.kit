The byte range of the extended grapheme cluster (UAX#29) the collapsed caret
would delete in the given direction, but **only** when that cluster spans
more than one Unicode scalar (spec §5.11 / §5.10).

Returns [`None`] — leaving the buffer's default one-`char` deletion (and its
coalescing) in charge — when there is no active collapsed caret, nothing to
delete, or the cluster is a single scalar (the common case). A ZWJ emoji
sequence or a base+combining-mark cluster is many scalars but one cluster, so
this returns the whole `[start, end)` for the runtime to select and delete as
one edit.
