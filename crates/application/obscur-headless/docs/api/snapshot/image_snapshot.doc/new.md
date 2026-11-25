An explicit tolerance.

`max_channel_delta` is the per-channel noise floor; `max_differing_fraction`
is the share of pixels allowed to exceed it, clamped into `0.0..=1.0`.

A non-finite `max_differing_fraction` is clamped to `0.0`, not propagated:
`NaN` compares false against every threshold, so letting it through would
produce a golden check that silently accepts *everything*.
