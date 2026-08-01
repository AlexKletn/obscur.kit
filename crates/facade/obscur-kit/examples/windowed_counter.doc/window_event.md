Routes window events.

Every native event is forwarded to the backend's handler first, which
converts the §6.1 lifecycle ones (resize, close, focus, scale) into domain
[`DomainWindowEvent`]s for [`Self::drain_window_events`] to act on. What is
handled *here* is what the window port deliberately does not carry: pointer,
keyboard and IME input, plus the redraw that presents a frame.
