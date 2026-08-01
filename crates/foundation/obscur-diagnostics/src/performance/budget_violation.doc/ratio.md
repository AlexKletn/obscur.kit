Returns the measured/budget ratio (e.g. `1.5` means 50% over budget).

Returns [`f64::INFINITY`] for a zero budget with a non-zero measurement,
and `0.0` for a zero budget with a zero measurement.
