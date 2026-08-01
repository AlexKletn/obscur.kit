**Headless proof: the interactive Wireframe-Kit demo WORKS — and the bare
controls get their look entirely from the installed UA default stylesheet.**

The deterministic, display-free sibling of `wireframe_live_window.rs`. It boots
the shared [`wireframe`] scene on a real [`UiRuntime`] with the kit's **UA
default stylesheet installed** (`runtime.set_default_stylesheet(
obscur_kit::ua_stylesheet())`) — the ONLY thing that styles the bare controls —
plus the demo's LAYOUT-CHROME author sheet on top, then drives every interaction
**programmatically through the real input path** ([`handle_pointer`] for hover /
press / release / click, [`handle_key`] for typing) and rasterizes two stitched
panels to `target/wireframe_live_proof.png`:

- **Panel 1 — Idle.** The spec sheet at rest: masthead, seven section cards, and
  the footer, every bare control showing its neutral UA base look.
- **Panel 2 — Interacted.** The button hovered + pressed (`:hover` + `:active`
  + `:focus` all biting from the live pointer/press/focus targets), the field
  focused with typed text, the checkbox toggled `:checked` (box + `✓`), the
  tri-state advanced, one radio selected (filled dot), and the select opened +
  picked (arrow flip + chosen label).

Each interaction is located by hit-testing the runtime's own layout boxes (no
pixel positions are hard-coded), driven through the pointer/keyboard API, and
**asserted**, so a regression fails the run rather than producing a wrong image.
The disabled button is clicked and shown to be a genuine no-op.

Run it:
`cargo run -p obscur-kit --example wireframe_live_headless \
  --features "render-cpu text-harfrust text-unicode"`
