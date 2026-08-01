The §7.14 tolerance under test, with both knobs measured above.

Both knobs are load-bearing, and *independently* so. Tightening each to
nothing fails a **disjoint** set of tests here — measured, not assumed:

- `max_differing_fraction` → `0.0` fails exactly the two AA-band cases
  (`worst_case_anti_aliasing_variation_is_accepted` and
  `the_cap_does_not_catch_a_sub_pixel_radius_drift`); the precision case
  still passes, because nothing there exceeds the floor to be counted.
- `max_channel_delta` → `0` fails exactly
  `a_backend_precision_shift_inside_the_noise_floor_is_accepted`; the AA case
  still passes, because its band pixels differ by 240 either way.

The reverse direction does **not** separate, and no such claim is made:
loosening *either* knob alone (`255`, or `1.0`) waves through all four
rejection cases, because both knobs feed the one verdict — the floor decides
which pixels count, the cap decides how many may. Two permissions, one gate.
