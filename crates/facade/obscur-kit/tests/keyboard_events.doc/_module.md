The keyboard half of §2.3, reached the way an application reaches it.

Three things were wrong at once and they were one thing: there was no raw
keyboard *event*. Input was resolved straight to a command, so only four event
names existed and `keydown` was not among them; `on:keydown="…"` bound a hashed
symbol nothing ever dispatched. `prevent_default` was consulted for §2.6's
routing chain but not for the runtime's own default actions — and on the key
path the text edit ran *before* any dispatch, so refusing it was not merely
unimplemented but unreachable. And the whole body returned early when nothing
was focused, which put §2.6's own last two links — the window surface and the
application — out of the keyboard's reach.

These cases are written from the consumer's side, only `obscur_kit`'s public
API, because all three were delivery gaps and a delivery gap is only closed at
the boundary an application stands on.

Not a `spec_*` file: the spec-derived suites are an oracle written without
reading this repository. The §2.3 conformance statements live in
`spec_ch2_interaction.rs` and are not touched here.
