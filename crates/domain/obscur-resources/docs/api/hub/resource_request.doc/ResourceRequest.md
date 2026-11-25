A declarative intent to acquire a resource (spec §5.15).

A request names *where* a resource comes from ([`ResourceSource`]), *what*
category it is ([`ResourceKind`]), and the [`ResourceNamespace`] scope that
owns it for disposal and hot reload. Submitting a request to the hub yields a
[`crate::handle::ResourceHandle`]; the request itself performs no I/O.

# Deduplication

Two requests denote the **same** resource when they share the same source
*and* the same kind; the namespace is ownership metadata and does not affect
identity. The hub deduplicates on [`ResourceRequest::dedup_key`], so a second
request for an already-tracked `(source, kind)` returns the existing handle
and bumps its reference count rather than allocating a new resource.
