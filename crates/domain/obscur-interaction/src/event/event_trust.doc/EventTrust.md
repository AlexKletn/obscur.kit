§2.3's trusted/untrusted origin: whether an event was produced by a real
user gesture on a real device, or synthesized by something else.

# Why this is not decoration

This crate's [`PointerEvent`](crate::pointer::PointerEvent) is reachable from
three producers, and only one of them is a person: the winit adapter's
converter (a real OS event), `obscur_kit::devtools::apply_injected` (§7.6's
DevTools `Inject`, driven by whoever holds an `Inject`-scoped token), and the
headless harness's synthetic dispatcher (§7.4). Before this type the three
were **byte-identical** at the runtime's door, so an application could not
have declined to act on a gesture it never received.

# Fails closed

[`Untrusted`](Self::Untrusted) is the [`Default`], and
[`PointerEvent::new`](crate::pointer::PointerEvent::new) produces it. Trust is
*conferred* at the platform boundary by the adapter that received the OS
event — it is never assumed. A new injection path that forgets this type is
therefore untrusted, which is the harmless direction; the dangerous direction
(an injected event that passes for a real one) requires someone to write
[`Trusted`](Self::Trusted) on purpose.

A false negative — a real gesture marked untrusted, e.g. by a platform
adapter that never opts in — costs an application a legitimate action. That
is the trade this default makes deliberately.

# What this crate does *not* do with it

Nothing here gates on trust, and that is by design rather than an omission:
§7.6's agent loop exists to drive real clicks, so a runtime that refused
untrusted input would break the feature the flag describes. The runtime
records trust and reports it out through
[`PointerDispatch`](../../obscur_runtime/runtime/struct.PointerDispatch.html);
deciding that *this particular action* needs a real finger is the
application's call, because only the application knows which of its actions
are consequential.
