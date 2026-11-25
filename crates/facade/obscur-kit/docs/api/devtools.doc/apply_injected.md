Applies one authorized [`InjectedEvent`] (drained from the endpoint) to
`runtime` — the write half of the agent loop over the protocol (spec §7.6).

Pointer presses/releases map to a synthetic [`PointerEvent`] the runtime
hit-tests exactly like real input, so an external agent can drive clicks and
focus. Key injection is not yet mapped (a stable keycode → the runtime's
structured `KeyEvent` needs a keymap — a follow-up); such events are ignored.

# These events are untrusted, and must stay that way

Whoever holds an `Inject`-scoped token reaches this function, so nothing it
builds is a user gesture. Each event is left at
[`EventTrust::Untrusted`](obscur_interaction::event::EventTrust::Untrusted) —
[`PointerEvent::new`]'s default — and the runtime reports that out through
[`PointerDispatch::trust`](crate::runtime::PointerDispatch::trust),
which is what lets an application distinguish a driven click from a real one.

**Never call `PointerEvent::as_trusted` here.** That method exists for a
platform adapter holding an actual OS event; calling it on this path would
make injected input indistinguishable from a real gesture, which is the exact
condition this flag exists to rule out. `injected_pointer_is_not_trusted` in
`tests/devtools_live.rs` fails if it is ever added.

# Returns

The [`PointerDispatch`] the runtime produced, or [`None`] for an event this
path does not map (the key variants). Returning it is what makes the
untrusted mark *actionable*: `handle_pointer` reports trust only in its
return value — a direct call records nothing in
[`UiRuntime::take_input_dispatches`], which the queued path fills — so a
function that swallowed the dispatch would leave a host no way to tell that
the click it just applied came from an agent rather than a person.
