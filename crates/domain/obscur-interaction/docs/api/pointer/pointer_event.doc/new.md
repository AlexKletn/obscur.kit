Creates a pointer event with no buttons held, zero pressure, no movement,
no tilt or twist, a zero timestamp, and an
[`Untrusted`](EventTrust::Untrusted) origin.

Untrusted is deliberate, not a placeholder: a synthetic event that forgets
to say what it is must not pass for a real gesture. The one producer of
real input — the platform adapter holding the OS event — opts in with
[`as_trusted`](Self::as_trusted).

Two of §2.4's fields are defaulted from the *device* rather than from
nothing, because the device already answers them for the common case:
[`hover_capable`](Self::hover_capable) follows
[`PointerDevice::hovers_by_default`] (§2.4: *"Touch does not synthesize
persistent hover. Pen hover depends on device capability."* — so a pen
starts non-hovering and a platform that knows better opts in with
[`with_hover_capability`](Self::with_hover_capability)), and
[`primary`](Self::primary) starts `true`, which is correct for every
single-pointer host and for the *first* contact of a multi-touch one. A
producer that tracks several simultaneous contacts marks the others with
[`with_primary`](Self::with_primary)`(false)`.
