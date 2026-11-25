A progress bar: a fraction, and the shape it is drawn along.

The widget draws NOTHING itself. It emits a `<canvas>` naming one built-in
painter and carries the two numbers that painter needs, so every progress on
the screen shares one painter and differs only by its own properties.

Its local name is `progress`, which the accessibility projection already maps
to `SemanticRole::ProgressBar` — a role deliberately absent from the interactive
set, so Tab passes it by. A bar that reports rather than accepts should not be
a stop.
