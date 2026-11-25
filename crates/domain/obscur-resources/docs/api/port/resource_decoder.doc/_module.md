The hexagonal port for decoding a resource into a CPU representation (spec
§5.15).

This module defines [`ResourceDecoder`], the port that turns the raw encoded
bytes of a resource into a CPU-side representation (spec §5.15 decode) — for
an image, decoded RGBA pixels; for a font, a parsed face; for CSS, a parsed
stylesheet. Real decoders live in adapter crates (image, svg, font); at the
domain boundary the decoded representation is an opaque byte buffer plus its
[`ResourceMetadata`], carried by the colocated [`DecodedResource`] output. The
colocated [`NullDecoder`] is the honest default that supports nothing.

Decoding is modeled **synchronously** at the domain level: an adapter that
decodes on a worker thread wraps its own async and calls this port with the
result.
