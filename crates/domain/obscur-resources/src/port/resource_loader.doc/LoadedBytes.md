The raw encoded bytes of a resource plus its descriptive metadata (spec
§5.15).

This is the output of [`ResourceLoader::load`]: the still-encoded bytes (a
PNG blob, a font file, a CSS string) held in an [`Arc<[u8]>`] so clones are
cheap and the bytes can be shared across the decode/compile pipeline, paired
with the [`ResourceMetadata`] the loader could determine (at minimum the byte
size).
