winit's whole modifier state to the kit's chord.

A pure function over a winit value, like the rest of this type, so it is unit
tested directly rather than through an event loop.

It exists because folding a chord from key transitions is only correct while the
window keeps focus. A modifier released elsewhere produces no `KeyboardInput`
here, and the fold keeps believing it is held; `ModifiersChanged` is the
platform stating the truth, and `UiRuntime::set_modifiers` is where it lands.
