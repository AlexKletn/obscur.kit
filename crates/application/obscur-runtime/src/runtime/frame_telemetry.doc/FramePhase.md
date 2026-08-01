A frame phase that reports a deterministic cost count (spec §7.9), and the
three names that cost is recorded under.

The runtime is clock-free, so a phase has no duration to report; what it has
is a count of what it processed, and that count means different things in
different phases. Naming the phase — rather than passing a span name, a
counter label, a subsystem and an already-computed budget check — is what
keeps the four descriptions of one phase from drifting apart; see
[`FrameTelemetry::record_phase`].
