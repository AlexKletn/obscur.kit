A deterministic in-memory [`LazyDataProvider`](super::LazyDataProvider) reference implementation (spec
§6.20).

`StaticDataProvider` holds an ordered table of `(MimeType, bytes)` entries so
that [`Self::available_mimes`] reports them in insertion order and lookups are
reproducible. It is useful as a test double and as the honest provider for
offers whose lazy bytes are in fact already known. Entries are matched by
[`MimeType::matches`], so lookup is by format essence.
