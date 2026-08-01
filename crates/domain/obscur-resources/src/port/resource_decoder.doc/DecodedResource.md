A decoded CPU representation of a resource plus its metadata (spec §5.15).

This is the output of [`ResourceDecoder::decode`]: the decoded CPU bytes (for
example the RGBA pixels of an image) held in an [`Arc<[u8]>`] so clones are
cheap and the buffer can be shared with the compile stage, paired with the
[`ResourceMetadata`] the decoder determined — including the intrinsic pixel
size for a visual resource. The domain treats the buffer as opaque bytes; the
adapter that produced it knows its layout.
