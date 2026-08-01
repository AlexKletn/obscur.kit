A CSS environment value — `env(safe-area-inset-top)` and friends — in
logical pixels (spec §6.15).

Unlike every other term here it is not a coefficient times a base: the
host reports a distance, and the declaration *is* that distance. It
resolves from [`LengthContext::environment`], which is all zeroes for a
host that reports nothing — the correct answer for a screen with no
cutout and no on-screen keyboard.
