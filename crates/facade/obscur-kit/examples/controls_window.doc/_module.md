**Windowed interactive control gallery → Vello (GPU, live).**

A live windowed gallery of the base HTML controls where hovering, clicking,
focusing, typing, toggling, and opening the dropdown drive **real** state and
live CSS pseudo-classes recomputed by the runtime from the genuine input path.
It renders the shared [`controls`] scene through winit 0.30 + Vello/wgpu into
an on-screen window, and — unlike the two existing windowed demos
(`showcase_window.rs` wires only three controls; `uikit_window.rs` only swaps
themes) — it wires **every** genuinely-interactive control:

- **Button** — clicking fires `EVENT_CLICK` → `CMD_BTN`, bumping a click count
  shown in the status line; `:hover` / `:active` / `:focus` are recomputed live.
- **Checkbox** — a plain box and a **tri-state** one; a click runs
  [`Checkbox::toggle`], and the reserved `CHECKED` attribute drives `:checked`.
- **Radio group** — composed clickable rows sharing one selected index; the
  chosen row carries `CHECKED` so `:checked` fills it.
- **Select** — the header toggles the real [`Select`] popup; while open, the
  option rows show, and clicking one selects + closes it.
- **Input** — clicking seats the caret + `:focus` ring; typing / IME flows
  winit → converters → the runtime editing session → live text.
- A **disabled** variant of each shows `:disabled` and ignores input (its
  command has no handler, so activation resolves to nothing).

The status line ("last event: … · clicks: N") updates every interaction so the
events are visible. The window is **device-scale aware**: the render scale is
the display's HiDPI `scale_factor` — **not tied to the window size** — so
resizing the window reflows the layout rather than zooming the whole UI up and
down, exactly like a real application. Any content taller than the window is
reached by the mouse wheel, and `Ctrl+=`/`Ctrl+-`/`Ctrl+0` apply a user zoom on
top (this is the exact driver shape reused from `showcase_window.rs`).

Run it (a display + a GPU adapter are required):
`cargo run -p obscur-kit --example controls_window --features full`

# Environment note

Like `showcase_window.rs`, this needs a display and a usable GPU adapter; in a
headless environment it compiles but exits cleanly on `resumed` when no
adapter/surface can be acquired (it never panics on that path). The headless
CPU proof (`controls_headless.rs`) is the display-free verification.
