An opaque handle to a directory or container a file picker returned
(spec §6.10 — a handle is **not** necessarily a filesystem path).

Like a [`DocumentHandle`](crate::document::DocumentHandle), a directory handle
is keyed by an opaque `token` the host resolves, and works across local
folders, portals, cloud containers, and virtual filesystems. Its contents are
not stored here: enumerating the children is a
[`DocumentService`](crate::document::DocumentService) operation
(spec §6.10 enumerate).

The handle carries descriptive [`DocumentMetadata`] (whose `is_directory` flag
is conceptually true), the granted [`DocumentPermission`], and an optional
advisory `path_hint` that is *not* guaranteed to be a real filesystem path.
