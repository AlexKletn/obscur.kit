Returns the refresh rate in hertz, or `None` if unknown.

This is a convenience conversion of [`Self::refresh_millihertz`] and may
lose precision; the millihertz value remains the exact source of truth.
