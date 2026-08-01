§5.19 requires the cache metrics to be *reportable*, and a getter nothing calls
is not a report. The runtime's own `image_cache_statistics()` had no production
consumer: the DevTools `Caches` inspectable built its payload from the resident
counts alone, so an eviction reached no client and a host could not tell a cold
cache from a thrashing one.
