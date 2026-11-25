Returns how many line-metric queries this shaper has answered with the
idealized ratios ([`LineMetrics::idealized`]) because no real metrics were
reachable for the font asked about.

Non-zero means some line box on screen was sized by ratio, not by the
face. The fallback is by design — a shaper with neither wired bytes nor a
face table must still answer — but it is a degradation, so it is counted
rather than merely happening.

This counter is an **affordance, not a report**: nothing in the runtime
reads it, so a host that never asks still gets ratio-sized line boxes
without being told. Assert on it in a host's own text-stack test to pin
the wiring down.
