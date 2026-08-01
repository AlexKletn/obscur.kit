Only the invalidated nodes were re-resolved, in place. Every node absent
from `replaced` kept a style this frame did not recompute and therefore
provably did not change, so those are the only candidates — and `replaced`
holds the style each of them had before.
