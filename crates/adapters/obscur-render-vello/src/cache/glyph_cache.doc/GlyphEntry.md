A single cached glyph's rasterized coverage.

Holds the glyph's pixel dimensions and its coverage bytes behind a shared
[`Arc`] so a cache hit is a cheap pointer clone rather than a copy. The blob
is opaque here; the eventual atlas packer decides its exact layout.
