Advances every coasting container by `delta` seconds and returns the
distance each one's fling covered, for the caller to scroll by.

Empty when nothing is coasting, which is how the caller knows to schedule
nothing. Pair every non-empty result with
[`settle_flings`](Self::settle_flings).
