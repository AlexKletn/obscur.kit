A `Backspace`/`Delete`, with the extended grapheme cluster the caret would
delete already resolved when — and only when — that cluster spans more than
one Unicode scalar.

Grapheme-cluster deletion (spec §5.11 with §5.10 Unicode): one
Backspace/Delete removes one USER-PERCEIVED cluster (UAX#29) — a combining
sequence or a ZWJ emoji is many code points but one cluster. The pure
buffer approximates a cluster as one `char`, so when the caret's cluster
spans more than one scalar the runtime pre-resolves the whole cluster here
(the documented "layout-aware adapter drives the buffer with explicit
ranges" path) and deletes it as one selection edit. A single-scalar cluster
leaves `cluster` [`None`] and falls through to the buffer's default delete,
preserving deletion coalescing.
