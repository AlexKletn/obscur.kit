The replaceable port for reading the current power state (spec §7.11).

An adapter implements this trait once per platform and the domain holds it as
`Box<dyn PowerProvider>`. The trait is deliberately **object-safe** — no
generic or `Self`-returning methods — so providers can be stored
polymorphically.

Contract: a provider reports an honest [`availability`](Self::availability)
and never panics — a platform without power telemetry returns
[`SupportLevel::Unavailable`] and the plugged-in baseline profile.
