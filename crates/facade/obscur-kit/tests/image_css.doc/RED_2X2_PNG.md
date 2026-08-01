A real 2×2 opaque-red PNG (RGB, generated with `zlib`/`crc32`), embedded so the
test needs no image-encoding dependency: the fs loader reads these exact bytes
off disk and the `image`-crate decoder turns them into RGBA8.
