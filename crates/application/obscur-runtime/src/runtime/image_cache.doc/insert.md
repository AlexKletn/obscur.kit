Admits `image` for `url` under the budget, evicting as needed, and returns
the URLs evicted to seat it (spec §5.19 memory budgets, eviction).

`key` identifies the resource the pixels were decoded from and `version`
the version they were decoded at, so the bookkeeping can be traced back to
the hub entry that produced it.

An image too large to fit even an empty cache is **not** admitted and
nothing is evicted for it: throwing away images that do fit would not seat
one that never can. It simply does not paint, the same outcome a failed
load already has. Re-inserting a resident URL displaces the old entry and
counts it as an eviction (the live path cannot do this — a URL is requested
once — but the accounting must stay right if it ever does).
