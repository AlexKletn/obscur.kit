The host-reported environment values in force: safe-area insets and the
on-screen keyboard's geometry, which `env()` resolves against (spec
§6.15).

All zeroes until a host reports otherwise, which is the right answer for
a screen with no cutout and no keyboard rather than a placeholder for one.
