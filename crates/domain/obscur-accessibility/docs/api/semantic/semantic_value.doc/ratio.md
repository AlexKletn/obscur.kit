Returns the [`Number`](Self::Number) value normalised into `[0, 1]`.

The value is clamped to `[min, max]` before normalising. A degenerate
range (`min == max`) yields `0.0` rather than dividing by zero. Non-numeric
variants return `0.0`.
