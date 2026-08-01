A [`ResourceDecoder`] that validates SVG source with `usvg` and keeps it
vector (spec §5.17).

This decoder supports only [`ResourceKind::Svg`]. It parses the bytes as UTF-8
SVG (optionally gzip-compressed, which `usvg` handles), validates the
document, and returns the original source bytes as the [`DecodedResource`]
payload with the resolved intrinsic size in its metadata. Invalid SVG becomes
[`ResourceError::DecodeFailed`].
