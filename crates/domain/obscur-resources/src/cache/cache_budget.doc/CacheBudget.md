The memory budget and per-cache disable switch for one runtime cache (spec
§5.19 *memory budgets; per-cache disable switches for tests*).

A budget bounds a cache in two independent dimensions — total bytes and total
entry count — either of which may be unlimited (`None`). It also carries an
`enabled` flag: a **disabled** cache admits nothing, which lets tests turn an
individual cache off (spec §5.19) to exercise the uncached path deterministically.

This is pure policy data. It decides *whether* a store may hold more — via
[`admits`](CacheBudget::admits) before an insert and
[`exceeds`](CacheBudget::exceeds) to detect an over-budget cache that needs
eviction — but it never stores entries itself; that is the cache store's job
(a later sub-step).
