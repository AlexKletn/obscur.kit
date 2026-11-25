A [`ResourceDecoder`] that decodes raster images (PNG/JPEG) into RGBA8 pixels
via the `image` crate (spec §5.16).

This decoder supports only [`ResourceKind::Image`]; every other kind is
refused so the decoder composes honestly alongside the font and SVG decoders.
The decoded [`DecodedResource`] carries the raw RGBA8 bytes (four bytes per
pixel, `width * height * 4` in total) and metadata whose intrinsic size is the
image's pixel dimensions.
