A resource backed by bytes produced at runtime (spec §5.15 memory source,
§5.16 image bytes).

Unlike [`crate::source::EmbeddedSource`], whose bytes are baked into the binary
at build time, a memory source holds bytes that were produced at runtime — for
example decoded, downloaded by the host, or generated in process. It optionally
carries a MIME content-type hint the loader may use when interpreting them.

The bytes are held in an [`Arc<[u8]>`] so clones are cheap and two sources are
compared and hashed by their byte *content*, which lets the hub deduplicate
equal sources (spec §5.15).
