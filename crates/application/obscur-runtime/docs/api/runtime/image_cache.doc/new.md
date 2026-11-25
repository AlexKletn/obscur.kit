Creates an empty cache with the default budget — 128 MiB of decoded pixels and
512 images, roughly four full-screen 4K images or a wall of 256×256 thumbnails
— and the default [`EvictionPolicy`].

It is bounded rather than [`unlimited`](CacheBudget::unlimited) on purpose: an
unbounded default is a leak that only shows itself after the program has been
running for a while, which is the worst time to find it. A host with better
information replaces it through `UiRuntime::set_image_cache_budget`, and
`CacheBudget::unlimited()` is still there for anything that wants it.
