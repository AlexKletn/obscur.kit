Builds the counter UI from `count`: a dark `.counter` panel stacking a label
bar showing the value and the two real [`Button`]s, each rendered as a bar
that decrements/increments on click.

This is the reactive view BUILDER the runtime holds (installed in `new` via
[`UiRuntime::set_root_view`]); the runtime re-invokes it each reactive frame,
so passing the current `count` rebuilds the label after a click bumps it.
