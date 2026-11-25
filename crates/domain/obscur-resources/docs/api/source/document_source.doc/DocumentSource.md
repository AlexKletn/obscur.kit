A resource backed by a document the host resolves (spec §5.16 `DocumentHandle`,
§6.10 file pickers).

The concrete `DocumentHandle` type lives in the platform layer and is not a
dependency of this crate, so a document source models it abstractly: an opaque
`token` string the host resolves back to its document, plus an optional
`path_hint`.

The kit does not read the document itself here — this is a pure description.
A loader or host resolves the token to actual bytes later (spec §5.15: the kit
performs no arbitrary I/O by itself).

The `path_hint` is *not* necessarily a real filesystem path (spec §6.10): a
file picker may hand back only a display name or a sandboxed identifier, so the
hint is advisory metadata for display or logging, never something to open.
