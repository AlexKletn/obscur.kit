A deterministic [`PowerProvider`](super::PowerProvider) that returns a fixed profile, for tests
(spec §7.11).

`FixedPowerProvider` holds a [`PowerProfile`] supplied at construction and
always returns it from [`profile`](PowerProvider::profile), reporting
[`SupportLevel::Native`]. Tests use it to drive the domain with an exact,
reproducible power state.
