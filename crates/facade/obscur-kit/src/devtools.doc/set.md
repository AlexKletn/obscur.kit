Replaces the backing snapshot with `source` (called by the host while
[`has_client`](Self::has_client)).

A poisoned lock (a serving thread panicked mid-read) is recovered rather
than propagated, so a DevTools client fault never takes down the host.
