The **shared interactive control-gallery** scene, authored once and driven by
both the live windowed Vello example (`controls_window.rs`) and the headless
CPU→PNG proof (`controls_headless.rs`).

Like the capability showcase, it is deliberately **backend-pure**: it names
only the toolkit's view / widget / primitive / CSS authoring surfaces
(`obscur_kit::*`) and never a renderer, window, or GPU type, so the exact same
[`view`] + [`stylesheet`] feed a software rasterizer and a GPU surface
unchanged.

# What it exercises

A live gallery of the base HTML controls, each wired to real interaction state
so hovering, clicking, focusing, typing, toggling, and opening the dropdown
drive live CSS pseudo-classes recomputed by the runtime from the **real** input
pipeline:

- **Button** — `EVENT_CLICK` bumps a click counter; `:hover` / `:active` /
  `:focus` come straight from the runtime's pointer/focus/press targets.
- **Checkbox** — a plain [`Checkbox`] plus a **tri-state** one; a click runs
  [`Checkbox::toggle`], and the reserved `CHECKED` attribute drives `:checked`.
- **Radio group** — composed in the demo (there is no radio widget): a set of
  clickable elements sharing one selected index in state, the chosen one
  carrying the reserved `CHECKED` attribute so `:checked` fills it.
- **Select** — a real [`Select`] state machine ([`toggle_open`](Select::toggle_open)
  / [`select`](Select::select) / [`is_open`](Select::is_open)); the header
  toggles the popup and, while open, the option rows render from the widget's
  options. Because the widget emits a single bubbled `EVENT_CLICK`, the demo
  attaches a per-option click command so a windowed hit-test can tell which
  option was chosen — noted as composed, not a widget capability.
- **Input** — an editable [`Input`]; a click seats the caret + `:focus` ring,
  and typing / IME flows through the runtime's editing session, whose
  per-keystroke handler mirrors the text back into state.
- A **disabled** variant of each control shows `:disabled` (muted + dashed) and
  is a genuine no-op: its `EVENT_CLICK` binds a command with **no** registered
  handler, so the runtime's activation resolves to nothing.

# Open state

There is no reserved `OPEN` attribute / `:open` pseudo-class in the engine, so
the select's open state is expressed **structurally**: the option rows are in
the tree only while the real [`Select::is_open`] is true. Toggling the popup is
therefore a genuine state change driven by the widget's own state machine.
