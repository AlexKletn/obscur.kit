A named scope that groups resources for bulk disposal and scoped hot reload
(spec §5.15).

The Resource Hub tracks every resource under a namespace so a whole group can
be torn down in one call (a closed document, an unloaded plugin) or reloaded
together during development. A namespace pairs an opaque [`NamespaceId`] — the
identity the hub keys on — with an optional human-readable `label` used only
for diagnostics; equality and hashing follow the full pair.

[`NamespaceId::from_raw`]`(0)` is reserved for the implicit *global* namespace
that owns every resource requested without an explicit scope; see
[`ResourceNamespace::DEFAULT`].
