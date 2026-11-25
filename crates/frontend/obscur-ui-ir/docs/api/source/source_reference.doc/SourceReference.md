A span within a named source (spec §2.5).

Pairs a source identifier (`source` — a path, logical name, or other id)
with a `[start, end]` range of [`SourceLocation`]s. `start` is expected to
precede or equal `end` by byte offset; construction does not reorder them,
and [`SourceReference::byte_len`] saturates rather than underflowing if the
invariant is violated.

The source name is stored as a `Box<str>` to keep each reference compact and
immutable once built.
