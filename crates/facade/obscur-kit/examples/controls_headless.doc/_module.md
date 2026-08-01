**Headless proof: the interactive control gallery WORKS without a display.**

The deterministic, display-free sibling of `controls_window.rs`. It boots the
very same shared [`controls`] scene on a real [`UiRuntime`], then drives every
interaction **programmatically through the real input path** —
[`handle_pointer`](UiRuntime::handle_pointer) for hover / press / release /
click and [`handle_key`](UiRuntime::handle_key) for typing — and rasterizes the
resulting states through the software [`CpuRenderer`] into three stitched
panels written to `target/controls_gallery_proof.png`:

- **Panel 1 — Idle.** The gallery at rest: nothing hovered, focused, or
  toggled.
- **Panel 2 — Hover · Active · Open.** The checkboxes toggled `:checked`, the
  radio group with one option `:checked`, the **Select popup open** (its option
  list rendered from the real [`Select::is_open`] state), and the primary
  button held **pressed** so `:hover` + `:active` + `:focus` all bite from the
  runtime's live pointer/press/focus targets.
- **Panel 3 — Typed · Selected.** The button released (click count bumped), the
  Select collapsed onto its chosen option, and the text field **focused with a
  caret** showing text the run typed through the editing session.

Each interaction is located by hit-testing the runtime's own layout boxes (so
no pixel positions are hard-coded), driven through the pointer/keyboard API,
and asserted, so a regression fails the run rather than producing a wrong
image. The disabled button is clicked and shown to be a genuine no-op.

Run it:
`cargo run -p obscur-kit --example controls_headless \
  --features "render-cpu text-harfrust text-unicode"`
