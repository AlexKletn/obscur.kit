A real 2×2 opaque-red PNG, the same fixture `image_css.rs` uses: the fs loader
reads these exact bytes off disk and the `image`-crate decoder turns them into
RGBA8. Real bytes matter here — the load must be real work, not a mock that
returns instantly enough to hide where it ran.
