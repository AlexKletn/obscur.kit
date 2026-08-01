The per-node computed-style cache: [`StyleCache`] (§4.3, §5.19).

[`StyleCache`] memoises each node's [`ComputedStyle`] keyed by [`NodeId`], so
startup and subsequent frames can reuse an unchanged node's style instead of
recomputing it. Entries are version-stamped: a lookup that supplies a newer
version than the stored entry misses, keeping stale styles out of the results
(§4.3). Per §5.19 the whole cache has a disable switch — a
[`disabled`](StyleCache::disabled) cache never stores and always misses — so
correctness tests can run the resolver with caching transparently turned off.
