A canvas painter, registered the way an application is supposed to register
one, and the pixels that proves it arrived.

Two claims live here, one layer apart.

The **application's** painter: `-obscur-painter` names a routine by name, and
until `App::painter` existed there was no supported way to supply one — the
only door was `UiRuntime::register_named_canvas`, which an application that
uses `App` never holds. A `<canvas>` therefore painted nothing unless the host
abandoned the shipped entry point and drove the runtime by hand.

The **toolkit's own** painters: `Progress` and `Slider` build a `<canvas>`
naming a painter by symbol and draw nothing themselves, and nothing anywhere
registered those painters. A progress bar built through the shipped builder
laid out, clipped and reported itself to the accessibility tree, and put no
ink on the screen. That is the same defect one layer up, and
`KitBootstrap::install_builtin_components` — which every shipped runner
already calls — now closes it.
