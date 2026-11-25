A reference [`PowerProvider`](super::PowerProvider) for platforms without power telemetry
(spec §7.11).

`NullPowerProvider` honestly reports [`SupportLevel::Unavailable`] and a
profile whose source is the typed [`PowerSource::Unknown`] — a reading that
is deliberately *distinguishable* from a genuine plugged-in ([`PowerSource::Ac`])
one, so callers cannot mistake absence for a real state (§7.11 / README core
principle 7: unsupported operations return typed results, not fabricated ones).
Its answers are correct behavior — a documented reference for the
honest-capability contract — not a stub waiting to be filled in.
