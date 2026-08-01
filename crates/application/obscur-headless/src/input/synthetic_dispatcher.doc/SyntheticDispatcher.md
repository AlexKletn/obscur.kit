Replays a [`SyntheticInput`] against a live runtime and reports what it did
(spec §7.4).

This is the write half of headless testing. The descriptors and the resolver
only *describe* and *locate*; this drives.

# It uses the real path, deliberately

A dispatch is not a shortcut into the runtime's internals: a pointer input
becomes a [`PointerEvent`] aimed at a viewport point and goes through
[`UiRuntime::handle_pointer`] — the same call a windowed host makes from a
winit event. So hit-testing, the hover chain, focus movement and command
dispatch all behave exactly as they would for a real user, and a headless
assertion cannot pass while the real thing is broken. Aiming a `test-id` at
the centre of its box is the only synthesis; everything after is the engine.

This also generalises what every headless demo hand-rolled: locate a node,
compute a point inside it, build the event, call the runtime.
