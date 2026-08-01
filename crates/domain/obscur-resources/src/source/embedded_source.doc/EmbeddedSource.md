A resource whose bytes are baked into the binary at build time (spec §5.15
embedded source).

An embedded source names a logical asset and carries the immutable bytes that
were compiled into the program (for example via `include_bytes!`). It is a
pure description: the bytes are already in hand, so no loader, host, or I/O is
needed to resolve them.

The bytes are held in an [`Arc<[u8]>`] so clones are cheap and two sources are
compared and hashed by their byte *content*, which lets the hub deduplicate
equal sources (spec §5.15). This differs from [`crate::source::MemorySource`],
whose bytes are produced at runtime rather than baked at build time.
