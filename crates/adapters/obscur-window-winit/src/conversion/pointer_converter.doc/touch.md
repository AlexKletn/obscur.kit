Converts one winit touch `contact` into the domain [`PointerEvent`] for
that finger (spec §2.4 "the pointer model supports mouse, touch, pen and
trackpad-like sources", §4.7).

This is the whole of the touchscreen's path into the kit. Everything above
it — hit-testing, hover, focus, activation, capture — is already
device-agnostic and already reads [`PointerDevice`], so a finger needs no
parallel pipeline; it needs this conversion and nothing else.

The mapping:

- **Phase → kind.** `Started`/`Moved`/`Ended` are the pointer's
  `Down`/`Move`/`Up`. `Cancelled` is [`Cancel`](PointerEventKind::Cancel),
  *not* `Up`: the system took the sequence away (a gesture won it, the
  window lost the contact), and a cancelled contact must not activate what
  it happened to be over.
- **Contact → [`PointerId::touch`]**, so several fingers stay several
  pointers and the first one is not mistaken for the mouse.
- **Contact → the primary button held.** A finger has no buttons, but the
  pointer model's whole point is that device-agnostic code works, and such
  code asks whether the primary button is down to mean "is this pointer
  engaged". Contact is that engagement, so it is reported as the primary
  button — the pointer-events convention — and released on `Ended` and
  `Cancelled` alike.
- **Force → pressure**, normalized to `0.0..=1.0`. A platform that does not
  report force (winit gives it on iOS/Windows/Web/Android only, so not on
  this workspace's Wayland target) leaves pressure `0.0`, which the domain
  already defines as "a device without pressure" — it is not a claim of a
  feather-light touch.

Like the mouse builders, the event is marked [trusted](Self::pressed): it
came from the compositor for a real contact.
