Per-scene counters surfaced to `DevTools` and performance telemetry
(spec §7.9).

These are **diagnostic metrics only**: they describe the shape of a
[`DisplayList`] (how many commands, draws, layers, and clips it contains)
and never bear on rendering correctness. A renderer must not depend on them;
they exist purely for observability.
