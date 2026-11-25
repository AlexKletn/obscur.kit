A single pointer event delivered to a hit-tested target (spec §2.4, §4.7).

Each event describes one pointer's state at one instant. §2.4 enumerates what
that state is, and this struct is that list:

> The pointer model supports mouse, touch, pen and trackpad-like sources:
>
> - pointer ID;
> - device type;
> - buttons;
> - logical position and delta;
> - pressure;
> - tilt;
> - rotation;
> - primary-pointer status;
> - hover capability.

So: which [`pointer`](Self::pointer) and [`device`](Self::device) produced it,
the [`kind`](Self::kind) of change, the [`position`](Self::position) in the
target's coordinate space and the [`delta`](Self::delta) that brought it there,
the [`buttons`](Self::buttons) held *after* this event, the contact
[`pressure`](Self::pressure) (`0.0` for hover or a device without pressure),
the stylus [`tilt`](Self::tilt) and [`rotation`](Self::rotation), whether this
is the [`primary`](Self::is_primary) pointer, whether the device
[`hovers`](Self::hover_capable), and a logical [`timestamp`](Self::timestamp)
supplied by the caller — this crate never reads a clock.

**Four of those were absent** until the §2.3/§2.4 delivery pass: delta, tilt,
rotation and primary-pointer status were listed by the spec and modeled
nowhere, so no handler could have read them however the event reached it. They
default to "nothing to report" ([`PointerDelta::ZERO`],
[`PointerTilt::UPRIGHT`], no twist, primary) so a producer that knows nothing
about them keeps working unchanged and one that does opts in field by field.

It also carries §2.3's trusted/untrusted origin ([`trust`](Self::trust)).
[`new`](Self::new) produces an [`Untrusted`](EventTrust::Untrusted) event:
trust is conferred at the platform boundary by [`as_trusted`](Self::as_trusted)
and is never assumed. See [`EventTrust`] for why that direction.
