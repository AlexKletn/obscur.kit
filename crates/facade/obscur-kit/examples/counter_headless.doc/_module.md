End-to-end **headless** capstone for the `obscur-kit` facade.

This example drives the whole toolkit through its public umbrella crate only
(`obscur_kit::*`) — no individual crate is named — with **no display and no
GPU**. It is deterministic and CI-friendly: every stage is exercised through
the real pipeline and guarded with `assert!`/`assert_eq!`, so a regression
fails the run rather than printing wrong numbers.

It walks the full authoring-to-scene path in five acts:

1. **Styling frontend** — author a small CSS stylesheet and compile it via
   the CSS frontend (`source -> parse -> compile`), proving the frontend
   composes and surfaces unsupported features as diagnostics rather than
   crashing.
2. **Components tier** — build a "counter" UI (a container tagged with the
   `counter` class, holding a text label and two buttons) from the
   primitives + widgets tier.
3. **Pipeline** — boot a deterministic [`HeadlessHost`] via `KitBootstrap`
   and render one frame with the compiled stylesheet through
   `reconcile -> selector-match -> style -> layout -> scene`. The `.counter`
   rule matches the tagged container, so the frame paints a real fill: the
   display list is asserted non-empty.
4. **Determinism** — render the identical view + stylesheet again and assert
   the display-list commands are byte-identical.
5. **State change** — drive a [`Button`]'s headless state machine (press +
   release), rebuild the view, render again, and show the frame id advanced.

Run it with:
`cargo run -p obscur-kit --example counter_headless`
