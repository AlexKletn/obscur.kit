How many bytes of decoded pixels the cache holds before it starts evicting,
with no host budget installed: 128 MiB.

Unlimited is not a default a long-running program can survive — a file manager
that has shown a thumbnail for every file it ever walked past holds every one
of them. But a budget small enough to thrash is worse than none, so the number
is derived from what one screen actually costs: a full-screen 4K image is
`3840 × 2160 × 4` = 31.6 MiB of premultiplied RGBA, and 128 MiB seats four of
them — the screen, plus room to have scrolled.

It pairs with [`DEFAULT_MAX_ENTRIES`]: the two caps meet exactly at a 256×256
RGBA thumbnail (256 KiB), so 512 of those is 128 MiB and neither cap is the
one that always bites first.

A host that knows better says so with `UiRuntime::set_image_cache_budget`, and
`CacheBudget::unlimited()` is still available to anything that genuinely wants
the old behaviour.
