The one line of glue that lets a registered face be the host's bytes rather
than a copy of them.

A peniko [`Blob`] holds `Arc<dyn AsRef<[u8]> + Send + Sync>`. The kit's font
registry holds every face as `Arc<[u8]>`, and `Arc<[u8]>` cannot be coerced to
that trait object — `[u8]` is already unsized, so there is no second unsizing to
apply. Wrapping it in a sized newtype gives the coercion something to bite on,
at a cost of one pointer-sized allocation per registered face.

What it buys: `Blob::from(Vec<u8>)`, which is what this used to call, copies the
whole face. The face bytes are already resident for the life of the process —
`FontRegistry` never drops a registered or discovered face — so that copy was a
second permanent copy of a permanent table, and for a CJK face on this desktop
that is 26 MB apiece. Now the two hold the same allocation.
