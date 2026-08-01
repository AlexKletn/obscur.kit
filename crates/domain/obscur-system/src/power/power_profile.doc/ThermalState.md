Thermal pressure on the device, ordered least → most severe (spec §7.11).

Ordered `Nominal < Fair < Serious < Critical` so the derived [`Ord`] supports
the `>= Serious` threshold in
[`PowerProfile::is_constrained`](PowerProfile::is_constrained).
