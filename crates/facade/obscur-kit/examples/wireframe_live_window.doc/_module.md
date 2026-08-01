**Windowed interactive Wireframe-Kit demo → Vello (GPU, live).**

A live winit 0.30 + Vello/wgpu window laid out like the imported *Browser
Elements Wireframe Kit* — a graph-paper page, a masthead card, and seven
numbered section cards — driving **BARE** form controls that get their entire
look from the kit's **UA default stylesheet**. The driver installs that sheet
with `runtime.set_default_stylesheet(obscur_kit::ua_stylesheet())`, so hovering,
clicking, focusing, typing, toggling, and opening the dropdown all show the
wireframe base look with live CSS pseudo-classes recomputed by the runtime from
the genuine input path:

- **Button** — clicking fires `EVENT_CLICK` → `CMD_BTN`, bumping a click count in
  the status line; `:hover` (fill shift) / `:active` (invert) / `:focus` (dashed
  ring) are recomputed live. A disabled button ignores input.
- **Text Field / Textarea** — clicking seats the caret + `:focus` ring; typing /
  IME flows winit → converters → the runtime editing session → live text.
- **Select** — the header opens the list (arrow ▲); a click while open picks the
  next option and closes (arrow ▼, chosen label). A disabled select ignores input.
- **Checkbox** — a plain box, a tri-state box, and a disabled box; a click runs
  [`Checkbox::toggle`], and the reserved `:checked` fills the box with the `✓`.
- **Radio** — a live group of three sharing one selection (filled dot), plus a
  disabled one.
- **Text Link** — a bare `<a>` showing the UA underline + `:hover` bold.

The demo's OWN stylesheet is **layout chrome only** (masthead / cards / captions
/ graph-paper page) — it never restyles a control, so the UA sheet is the whole
source of the control look. The status line ("last event: … · clicks: N") updates
every interaction. The window is **device-scale aware**: the render scale is the
display's HiDPI `scale_factor` — **not tied to the window size** — so resizing the
window reflows the layout instead of zooming the whole UI up and down. The mouse
wheel scrolls, and `Ctrl+=`/`Ctrl+-`/`Ctrl+0` zoom on top (the exact driver shape
reused from `controls_window.rs` / `showcase_window.rs`).

Run it (a display + a GPU adapter are required):
`cargo run -p obscur-kit --example wireframe_live_window --features full`

# Environment note

Like the other windowed demos this needs a display and a usable GPU adapter; in
a headless environment it compiles but exits cleanly on `resumed` when no
adapter/surface can be acquired (it never panics on that path). The headless CPU
proof (`wireframe_live_headless.rs`) is the display-free verification.
