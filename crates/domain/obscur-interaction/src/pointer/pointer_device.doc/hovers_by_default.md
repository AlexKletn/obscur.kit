Returns §2.4's default **hover capability** for this device: whether it can
be *over* a target without touching it.

§2.4 rules two of the four cases outright — *"Touch does not synthesize
persistent hover. Pen hover depends on device capability."* — so
[`Touch`](Self::Touch) is `false` and [`Pen`](Self::Pen) is `false` **by
default only**: a pen's answer is a property of the tablet, not of the
enum, so a platform that knows its digitizer reports hover says so per
event with
[`PointerEvent::with_hover_capability`](super::PointerEvent::with_hover_capability).
Defaulting the pen to `false` fails in the direction that costs a hover
highlight rather than the one that invents a hover the device never
reported.

A [`Mouse`](Self::Mouse) and a [`Trackpad`](Self::Trackpad) drive a resting
cursor, so both hover by construction.
