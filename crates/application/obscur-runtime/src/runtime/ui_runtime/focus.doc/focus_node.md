Moves focus programmatically, or clears it with `None`.

The setter that was missing: focus had exactly one mover, the pointer press, so
an application could not put focus on the field it had just revealed or take it
back after a dialog closed.

A node the tree does not have is refused rather than accepted and quietly
dropped on the next frame. The move is `EventOrigin::Synthetic`, so it fires
`blur` / `focus` like any other and does NOT show a focus ring — the ring says
"you are navigating by keyboard", which a programmatic move is not.
