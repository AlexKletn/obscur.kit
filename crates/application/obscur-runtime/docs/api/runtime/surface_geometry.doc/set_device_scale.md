Sets the device-pixel scale, clamped to [`MIN_DEVICE_SCALE`] (a
non-positive or non-finite request is ignored in favour of the clamp).

Returns whether the change crosses into a different **DPI-variant band** —
i.e. whether it now selects different asset files (spec §5.16). A `2.5` →
`3.0` nudge (both `@3x`) reports `false`; a `1.0` → `2.0` display change
reports `true`. Reporting rather than acting is deliberate: the caches that
would have to be dropped are not this type's.
