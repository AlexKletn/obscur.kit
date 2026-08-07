How many entries the two URL-keyed bookkeeping maps hold, as
`(image_handles, image_referrers)`.

Test-only: no shipped surface reports it, and that is precisely why these two
could grow without bound unnoticed — `cached_counts` reports the caches, which
were never the maps at fault. It exists so a test can state the property
[`retain_referenced`](Self::retain_referenced) is for as an assertion about
sizes, which is what a leak actually is.
