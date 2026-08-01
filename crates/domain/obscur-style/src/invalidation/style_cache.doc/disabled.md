Creates a disabled cache that never stores and always misses (§5.19).

Use this in correctness tests to run resolution as if the cache were not
present, without changing any call site.
