The `tile` rect of the single painted image — the box the image occupies in
logical pixels.

This is the read-side proof for DPI variants (spec §5.16): a `@2x` asset carries
four times the pixels and must still occupy the *same* logical box, so the tile
is where doubling would show up.

Panics if the list paints no image, because a test that reads a tile has already
decided one was painted.

# Panics

If the list paints no image. A test that reads a tile has already decided one was
painted, so an empty list is a broken premise rather than a missing value.
