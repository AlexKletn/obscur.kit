An envelope carrying a wheel notch, under the pointing device's origin.

**Untrusted and unstamped, deliberately.** [`from_pointer`](Self::from_pointer)
and [`from_key`](Self::from_key) report what their device event reported, because
a `PointerEvent` and a `KeyEvent` are self-describing and a platform converter
marked them. A wheel has no such door: `UiRuntime::handle_scroll` takes a delta
and a position, so nothing at the boundary knows whether this notch came from a
device or from a queued `InputEvent::Scroll`. [`EventTrust`](super::EventTrust)
fails closed, and a host that does know says so with
[`as_trusted`](Self::as_trusted) and [`with_timestamp`](Self::with_timestamp).
