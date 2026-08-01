The supersample render scale: the `UIKIT_SCALE` env var parsed as a float and
clamped to `1.0..=4.0` (default `2.0`), then further capped so `VIEWPORT × scale`
fits [`MAX_TEXTURE_DIM`] on both axes (the sheet is tall enough that a full 2×
overflows the GPU texture ceiling).
