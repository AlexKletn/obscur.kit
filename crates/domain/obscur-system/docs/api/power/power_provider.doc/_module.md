The replaceable port for reading platform power state (spec §7.11).

[`PowerProvider`](super::PowerProvider) is the hexagonal port through which the domain asks the host
for the current [`PowerProfile`] and learns, via
[`availability`](PowerProvider::availability), whether the platform reports
power information at all. Capability is honest: a platform without power
telemetry answers [`SupportLevel::Unavailable`] and reports a typed
[`PowerSource::Unknown`] rather than fabricating a plugged-in reading. The
colocated [`NullPowerProvider`](super::NullPowerProvider) and [`FixedPowerProvider`](super::FixedPowerProvider) are reference
implementations of that contract.
