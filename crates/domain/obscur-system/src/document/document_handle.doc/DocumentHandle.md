An opaque handle to a single document a file picker returned
(spec §6.10 — a handle is **not** necessarily a filesystem path).

The handle works uniformly across local files, portals, cloud documents, and
virtual filesystems: its primary key is an opaque `token` the host resolves
back to the underlying document. The kit never reads the document itself here
(spec §5.15) — a [`DocumentService`](crate::document::DocumentService) adapter
resolves the token to bytes later.

Alongside the token, a handle carries descriptive [`DocumentMetadata`], the
granted [`DocumentPermission`], and an optional `path_hint`. The path hint is
**advisory only** (spec §6.10 optional path hint): a picker may expose only a
display name or a sandboxed identifier, so the hint is metadata for display or
logging and is *not* guaranteed to be a real filesystem path — never open it.
