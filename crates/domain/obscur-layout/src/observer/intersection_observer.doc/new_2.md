Creates an observer for `node` with the given `threshold`, clamped to
`0.0..=1.0`. It has not yet observed intersection, so the first
[`observe`](Self::observe) fires with the initial value.
