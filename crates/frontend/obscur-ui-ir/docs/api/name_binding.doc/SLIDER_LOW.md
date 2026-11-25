Reserved slider value (`obscur_view::view::SLIDER_LOW`).

The authored spelling is `value`, and it binds to the *low* end because a single
slider is a range of one point: `SliderControl` reads this end and takes the
high one to be the same when no `SLIDER_HIGH` is stated. That is why an authored
slider needs one attribute and not three.

It is the symbol the runtime's drag looks for to decide that a node *is* a
slider at all (`SliderControl::enclosing` walks ancestors until it finds it), so
a hashed `value` would produce a control no pointer could grab.
