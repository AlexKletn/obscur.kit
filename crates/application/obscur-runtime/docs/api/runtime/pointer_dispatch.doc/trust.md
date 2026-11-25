§2.3's trusted/untrusted origin, carried through from the
[`PointerEvent`](obscur_interaction::pointer::PointerEvent) that produced
this dispatch.

[`Trusted`](EventTrust::Trusted) for input a driver vouched for: a platform
adapter converting a real OS event, and the first-party **headless** test
harness (§7.4) standing in for a real user. [`Untrusted`](EventTrust::Untrusted)
for input no one vouches for: the public programmatic
[`handle_pointer`](super::UiRuntime::handle_pointer) API called with a bare
event, and an external DevTools `Inject` (§7.6) arriving over the socket.

The runtime **gates the default action** on this (§2.3 step 4): a click's
activation — its bound command and handler — runs only for a trusted
gesture, so an untrusted event still dispatches and is observed (it
hit-tests, and hover/focus still move) but resolves
[`command`](Self::command) to `None` and fires no handler. This keeps host
control over a consequential action behind a real gesture (§1.8, §9): an
external agent may drive observation, not the gated action.
