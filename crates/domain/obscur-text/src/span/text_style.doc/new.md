Creates a weight from a raw value, clamped to the valid `1..=1000` axis.

Values of `0` or above `1000` are clamped rather than rejected, so this
never panics.
