A reference, in-memory [`DocumentService`](super::DocumentService) — a deterministic virtual
filesystem (spec §6.10).

`MemoryDocumentService` is the honest headless default: it stores document
bytes keyed by their opaque token and directory membership as token lists,
with no real I/O, clock, or randomness. It exists to exercise the port and to
give tests and headless runs a working document backend.
