The decoded images resolved from `background-image: url(...)` and element
`src`, keyed by the raw authored URL/path string, handed to the scene
coordinator each compose. A URL is inserted once its bytes load and decode;
a failed load caches nothing (the image simply is not painted).

Decoded pixels are the frame's largest per-URL cost, so this is the cache
the §5.19 budget/eviction/metrics tier is applied to — see [`ImageCache`]
and [`set_image_cache_budget`](Self::set_image_cache_budget). Unbudgeted by
default.
