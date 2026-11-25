The current power state feeding adaptive quality (spec §7.11).

Constructed from the plugged-in baseline via [`new`](Self::new) / [`Default`]
and refined with the fluent `on_source` / `with_battery_level` /
`low_power_mode` / `with_thermal` setters. The runtime calls
[`is_constrained`](Self::is_constrained) to decide whether to lower quality.
