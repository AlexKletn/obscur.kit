Resolves the [`KeyframeTrack`] a *superseding* transition should actually
run, given the incoming `target` track, the element's `current` sampled
value and the interrupted transition's `origin` value (spec §4.17
"interruption / reversal / blend").

Returns [`None`] only for [`Ignore`](Self::Ignore) — the signal to keep
the in-flight transition and drop the incoming one. Otherwise:

- [`Restart`](Self::Restart): the incoming `target` track verbatim (a
  fresh play from its authored start; the element may visibly jump).
- [`Blend`](Self::Blend): `target` re-based to begin at `current` via
  [`KeyframeTrack::with_start_value`], so the redirect is seamless.
- [`Reverse`](Self::Reverse): a two-stop track from `current` back to
  `origin` ([`KeyframeTrack::from_endpoints`]), undoing the in-flight
  transition from the present value.

Keeping this pure (no timeline, no clock, no CSS) lets both the runtime
and tests derive the redirect track deterministically.
