End-to-end **windowed** capstone for the `obscur-kit` facade.

Unlike `counter_headless.rs` (no display, no GPU), this example wires the
*real* platform backends the `full` feature set unlocks — winit 0.30 for the
window and event loop, and Vello/wgpu 29 for the GPU renderer — and drives
them through the toolkit's public umbrella crate only (`obscur_kit::*`). It
opens a genuine OS window and **presents the painted counter on-screen**,
redrawing on input and resize, running until the window is closed. Moving the
pointer over a bar lights its `:hover` style through the real input path, and
clicking the lower red bar decrements the counter while the green bar
increments it.

Run it on a real machine (a display + a GPU adapter are required):
`cargo run -p obscur-kit --example windowed_counter --features full`

# Wiring

1. **Window / event loop** — the window is created **through the toolkit's
   own §6 window port**, not by calling winit directly. The host issues
   [`WindowBackend::create_window`] on a
   [`WinitWindowBackend`] with domain [`WindowProperties`], and gets a
   [`WindowId`] back straight away; creation is *deferred*, so the OS window
   is materialized by the backend's [`WinitAppHandler`] on the next event-loop
   turn. This host therefore **wraps** that handler — every winit callback
   forwards to it first — and then reads what it left: the platform handle via
   [`WinitWindowBackend::native_window`] (an [`Arc<Window>`], which is what
   wgpu needs to build a `Surface<'static>`), and the window's lifecycle via
   the port's [`WindowBackend::poll_window_events`] — domain `Resized` drives
   the swapchain, domain `CloseRequested` exits.

   **Input is deliberately not on that path.** The port carries §6.1 window
   lifecycle, not keystrokes: pointer/keyboard/IME stay on the raw winit
   events, converted by the adapter's `PointerConverter`/`KeyboardConverter`.
   That split is the design, not a gap.

   **What this host still reaches around the port for** — stated because
   "created through the port" invites the reading that *everything* goes
   through it, and that is not true here:

   - `window.request_redraw()` — frame scheduling is §3.5, not §6. The port
     does have a path (`UserEvent::WakeUp` makes the handler redraw every live
     window); this host calls the handle directly because it already holds it.
   - `window.set_ime_allowed(true)` — §5.12 IME, not a §6 window property.
   - `window.inner_size()` / `window.scale_factor()` at startup — the port
     reports later changes as domain `Resized` / `ScaleFactorChanged`, but has
     no pull-based accessor for the *initial* extent.

   So the honest claim is narrower than "the host is backend-agnostic": window
   **creation, lifecycle and close** go through the port and name no winit
   type; presentation and input do not, and the `Arc<Window>` needed for the
   swapchain is fetched from the winit adapter, not the port. See
   `WinitWindowBackend::native_window` for why that handle is on the adapter.

   Beware one unit trap when copying this host: domain `Resized` carries
   **physical** pixels while `WindowProperties::size` is **logical**. This
   example is correct only because both coincide at scale 1 — see the known
   defect noted on `window_winit::conversion::WindowEventConverter::convert`.
2. **GPU device + surface** — wgpu `Instance → Surface → Adapter → Device →
   Queue`, with the device/queue handed to `VelloDevice::new` so the adapter
   never owns the async `request_adapter` path (spec §2.7). The surface is
   wrapped in a `VelloSurface` that owns the swapchain lifecycle.
3. **Frame driver** — the embeddable [`UiRuntime`] runs the real,
   *scheduler-gated* pipeline (reconcile → pseudo-aware match → style → layout
   → scene) and exposes the committed scene as a `SceneSnapshot`. Input,
   resize, and the counter's value each `request_frame` a reason;
   [`UiRuntime::run_due_frame`] then produces at most one frame per redraw
   when something is actually due, and the runtime retains the last frame's
   layout boxes so its hit-test index can map a pointer back to a node.
4. **Present** — the snapshot is packaged as a `RenderFrame` +
   `FrameSubmission` and submitted to `VelloRenderer::present_to_surface`,
   which rasterizes the scene into an offscreen Vello texture and then blits
   that texture onto the acquired swapchain image (Vello 0.9 has no
   render-to-surface helper; offscreen-render + blit is the documented
   pattern). The counter is therefore genuinely visible on-screen.

The runtime's pseudo-aware matcher matches the compiled `.counter` /
`.bar` / `:hover` stylesheet against the reconciled tree, so the committed
display list carries a real fill — that fill is what is rasterized and
presented.

# Input

Pointer input flows through the **real domain path**, not a geometric hack:
winit → [`PointerConverter`] → [`UiRuntime::handle_pointer`] → hit-test →
hover/focus → `:hover`/`:focus` restyle. Concretely:

- `CursorMoved` is converted by [`PointerConverter::moved`] into a domain
  [`Move`](obscur_kit::interaction::pointer::PointerEventKind::Move) event and
  dispatched; the runtime hit-tests it against the retained layout boxes,
  updates the hover chain, and (on a change) schedules an `Input` frame — so
  hovering a bar visibly swaps to its lighter `:hover` background.
- `MouseInput` (left press *and* release) builds
  [`Down`](obscur_kit::interaction::pointer::PointerEventKind::Down) /
  [`Up`](obscur_kit::interaction::pointer::PointerEventKind::Up) events —
  assembled directly with [`PointerConverter::kind`] mapping the winit state,
  since the converter has no public builder for a button transition *at a
  position* — and dispatches both. The `Down` anchors the click; the `Up` on
  the same node completes it, and [`UiRuntime::handle_pointer`] resolves it to
  the hit button's command.

The counter *value* is now driven through this **real command path** — no
geometric hack. Each [`Button`] binds its click to the reserved `EVENT_CLICK`
symbol carrying its own command (`CMD_INCREMENT` / `CMD_DECREMENT`); a
registered [`UiRuntime::on_command`] handler mutates a shared
`Rc<Cell<i32>>`, and the runtime's held view builder
([`UiRuntime::set_root_view`]) rebuilds the counter from the new value on the
next reactive frame. So clicking the green bar increments the on-screen count
and the red bar decrements it, and hovering a bar lights its `:hover` style —
both through the same domain input path.

# Text (multi-font fallback stack)

The counter's labels ("Count 数字: N", "-", "+") are real text with a
deliberately **mixed-script** label. Fonts are a host resource (like the GPU
device): `resumed` loads a primary system font plus (best-effort) a second,
CJK-covering fallback font off disk and wires the FULL fallback stack via
[`wire_text_stack`]:

- a [`UnicodeTextItemizer`] on the runtime ([`UiRuntime::set_text_itemizer`])
  splits each label into uniform bidi-level + script runs;
- a coverage-driven [`FontRegistry`] resolver
  ([`UiRuntime::set_font_resolver`]) picks, per run, the first registered font
  whose `cmap` covers it (both fonts registered under `sans-serif`, so
  COVERAGE — not name — disambiguates);
- a font-less [`HarfrustShaper`] ([`UiRuntime::set_text_shaper`]) shapes each
  item against its resolved face's in-memory bytes.

The renderer is handed the **same** id->bytes assignment via
[`VelloRenderer::register_font`], so the scene's per-run font handle (the
resolved [`FontId`](obscur_kit::text::font::FontId)) selects the matching font
when Vello rasterizes. On a machine with both fonts present, the Latin part
draws from the primary and the CJK part from the fallback. With no fallback
found the CJK glyphs render as `.notdef` (the fallback path activates only once
a covering font is registered); with no font at all, text is skipped.

# Environment note

This example cannot run in the headless dev/CI environment (no display, no
GPU adapter), so it is only ever *compiled* there. On a real machine it opens
a window, presents the painted counter, and responds to clicks until closed.
