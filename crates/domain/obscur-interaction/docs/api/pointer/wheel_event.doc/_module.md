A wheel notch: a delta and the point it was aimed at.

The third kind of input, and the reason it is not the first two. §2.4's pointer
model is a *contact* — an id, a device, buttons, pressure, tilt, a position it
moved to. A wheel has none of that. It is a request to scroll by an amount, at a
place, and the only two facts about it are the amount and the place.

Reusing [`PointerEvent`](super::PointerEvent) for it was considered and refused.
There is no `PointerEventKind::Wheel`, so a wheel would have had to arrive as a
`Move` carrying a delta — a report that the pointer moved when it did not, which
the model then cannot tell from a real move. The delta would ride in
[`PointerDelta`](super::PointerDelta), which means "how far the pointer travelled
since its last report", and it would there mean something else. One synthesised
lie is worse than one honest variant.

What it deliberately does *not* carry:

- **No `delta_x`.** `UiRuntime::handle_scroll` is vertical, so a horizontal field
  would be a number nothing ever fills — the same defect this campaign exists to
  close, one level down.
- **No trust, and no timestamp.** Both live on the envelope
  ([`UiEvent`](crate::event::UiEvent)) rather than here, because unlike a key or a
  pointer this event has no platform converter to confer them: `handle_scroll`
  takes two floats. See [`UiEvent::from_wheel`](crate::event::UiEvent::from_wheel)
  for what that means for a handler.
