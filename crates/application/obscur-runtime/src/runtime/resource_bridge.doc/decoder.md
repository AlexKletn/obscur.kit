The injected decoder that turns a loaded image's encoded bytes into RGBA8
pixels + intrinsic size (spec §5.16), or [`None`] when none was installed.
Both a loader and a decoder are required to resolve an image; either absent
means `url()` backgrounds paint nothing.
