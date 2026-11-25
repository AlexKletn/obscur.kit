The horizontal delta, in whatever [`mode`](WheelEvent::mode) says.

Carried but not acted on: this runtime scrolls vertically only. It is here
because the platform reports it and an application with a horizontal list needs
it — the winit runner used to discard it with a `_` pattern, so a two-finger
sideways swipe reached nothing at all.
