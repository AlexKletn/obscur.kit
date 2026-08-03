Routes a wheel notch stated as a bare delta and position.

Equivalent to [`handle_wheel`](Self::handle_wheel) with a
`WheelEvent::new(position, delta_y)` — untrusted, unstamped, pixels, no
horizontal component — which is exactly what two floats can say. A host that
knows more than that should build the event and use `handle_wheel`.
