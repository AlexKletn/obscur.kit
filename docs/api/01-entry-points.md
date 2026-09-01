# 01 — Entry points

Three ways in, in increasing order of how much you own yourself.

| | You supply | You own | Feature |
|---|---|---|---|
| [`App`](#app) | a view builder and handlers | nothing | `render-vello` + `window-winit` and/or `shell-layer` |
| [`UiRuntime`](#embedding-a-uiruntime) | the surface and the loop | everything around a frame | any |
| [`HeadlessHost`](#headless) | nothing | a deterministic driver | any |

`App` covers both an ordinary window and a **desktop surface** — a panel, dock,
bar or overlay. One call chooses:
[`App::layer(settings)`](#opening-a-desktop-surface-instead-of-a-window).

---

## `App`

`obscur_kit::App` — the one-call entry (`src/app/app.rs`). It owns the surface,
the GPU swapchain, the event loop, the text stack and the frame pipeline. It
needs `render-vello` plus at least one surface backend — `window-winit`,
`shell-layer`, or both — and is absent from the headless default build.

```rust
use obscur_kit::App;

fn main() -> Result<(), obscur_kit::AppError> {
    App::new("Hello")
        .with_size(480.0, 240.0)
        .stylesheet(".root { background-color: #202430; color: #eef }")
        .view(|| build_view())
        .run()
}
```

### The whole surface

Every method is a consuming builder returning `Self`, except `run`.

| Method | |
|---|---|
| `App::new(title: impl Into<String>) -> Self` | starts at **800×600** logical, app id `"obscur-kit"`, no styling, no root |
| `app_id(impl Into<String>) -> Self` | the Wayland `app_id` / X11 `WM_CLASS`. This — not the title — is what a compositor rule, a taskbar entry and a `.desktop` file match on. Give every app a distinct one |
| `with_size(f32, f32) -> Self` | initial size in **logical** pixels; the window stays resizable |
| `stylesheet(&str) -> Self` | appends a sheet. Callable repeatedly; later sheets cascade over earlier ones. This is the *initial* set — to replace it while running, see `control()` |
| `view(impl Fn() -> View + 'static) -> Self` | the root builder, re-invoked per frame |
| `view_with(impl FnMut(&FrameInfo) -> View + 'static) -> Self` | the root builder, given the live viewport and scale |
| `screen(&str) -> Self` | the root as `.okui` markup: compiled to UI IR at `run`, then re-lowered every frame against `screen_data()` |
| `screen_data() -> ScreenData` | the table a `screen`'s `{bindings}` read. Plain application state — write it before `run` and from a handler |
| `control() -> AppControl` | the handle onto this app's running runtime: positional handlers, listeners, `set_stylesheets`, a queued `with_runtime` escape hatch |
| `on_command(InternedSymbol, handler) -> Self` | a handler for a command name bound in the view or the sheet, at the **window** level (§2.6 link 4) |
| `on_command_at(SelectorTarget, InternedSymbol, handler) -> Self` | the same, **at a node** named by `test-id` (§2.6 links 1 and 3). Claims the command and stops the chain |
| `add_event_listener(SelectorTarget, InternedSymbol, ListenerPhase, listener) -> Self` | a §2.3 listener that observes and steers a command in `Capture` or `Bubble` |
| `on_scroll(impl FnMut(f32) + 'static) -> Self` | wheel deltas **no `overflow: scroll` container consumed** |
| `painter(impl Into<String>, painter) -> Self` | a canvas painter under the CSS-visible *name*, which is what `-obscur-painter: <ident>` reaches. See [04 — Styling](04-styling.md#vendor-properties) |
| `painter_id(InternedSymbol, painter) -> Self` | the same, under the *symbol* a node names in its own property — including a shipped widget's (`Slider::PAINTER`, `Progress::PAINTER`), which this replaces for the whole application |
| `virtual_list(InternedSymbol, impl FnMut(usize) -> ViewNode + 'static) -> Self` | the row builder for a list the runtime windows |
| `on_row_click(InternedSymbol, impl FnMut(usize, u32) + 'static) -> Self` | `(model index, click count)` for a row of that list |
| `on_row_long_press(InternedSymbol, impl FnMut(usize) + 'static) -> Self` | a held row of that list |
| `run(self) -> Result<(), AppError>` | opens the window and runs to completion |

`view`, `view_with` and `screen` set the same slot: **the last one called wins.**

### A handler takes nothing, or takes the event

`on_command`, `on_command_at` and `add_event_listener` accept either shape
(`IntoCommandHandler`):

```rust
.on_command(SAVE, || save())                              // takes nothing
.on_command(PICK, |e: &mut UiEvent| pick(e.target()))     // takes the event
```

The event form is how §2.3's model reaches an application: `target()`,
`current_target()`, `pointer()` / `key()`, `trust()`, and
`stop_propagation()` / `prevent_default()`. A one-argument closure needs its
parameter annotated — inference cannot pick the marker and the signature at once.
Until this widened, the handler was an `FnMut()` given nothing, so a handler had
to *capture* everything it might need.

`on_row_click`, `on_row_long_press` and `on_scroll` stay outside that model: the
runtime calls them straight from its pointer path rather than dispatching a
command, so no event is delivered and no listener sees them.

### Reaching the running runtime

The loop owns the runtime and your handlers run inside it, so nothing hands out a
`&mut UiRuntime`. `App::control()` is the way in — a cloneable handle a handler can
capture, whose requests the runner applies at the next frame boundary:

```rust
let control = app.control();
control.set_stylesheets(vec![theme_css()]);   // §4.12 explicit invalidation
// `with_runtime` takes `impl FnOnce(&mut UiRuntime) + 'static`, so the closure
// must return `()` — brace-and-semicolon anything that answers, as this does.
control.with_runtime(move |rt| { rt.scroll_into_view(node); });
control.bound_node(&SelectorTarget::test_id("save")); // did that name resolve?
```

`set_stylesheets` is the reason a theme switch no longer costs a rebuild: `App`
used to collect the authored sheets once and never re-supply them, and never yield
the runtime, so §4.12's "invalidated **explicitly**" was unreachable from a
standalone application.

### `FrameInfo`

```rust
pub struct FrameInfo {
    pub viewport: Size,  // logical (CSS) pixels
    pub scale: f32,      // device-pixel ratio
}
```

Handed to a `view_with` builder each frame. This is what lets an application
window its own list to what is visible instead of building a node per item — the
alternative to `virtual_list`, for when the app wants to own the arithmetic.

### `AppError`

`ScreenParse(Vec<String>)` · `ScreenCompile(Vec<String>)` · `EventLoop(String)` ·
`Window(String)` · `Gpu(String)`.

A GPU or window failure exits the loop cleanly and is reported here rather than
leaving a blank window: `run` returns the reason.

### What `App` wires for you

On `resumed`, in order: the UA stylesheet, the author sheets through the CSS
pipeline, image and vector resources, the blocking executor, the real text stack
over the host's fonts, the Vello renderer and surface, and the winit event
converters. Then per event: pointer/touch conversion, wheel routing, keyboard and
IME, motion advance, virtual-list windowing, and text entry offered per focus.

Two environment variables it reads:

- `OBSCUR_FRAME_STATS=1` — per-phase frame timings every 60 presented frames
- `OBSCUR_NO_VSYNC=1` — see [08 — Rendering](08-rendering.md)

---

## Opening a desktop surface instead of a window

One call. `App::layer(settings)` puts the same application on the **desktop**
rather than in the window stack: a panel, a dock, a bar, a wallpaper, a
notification, an on-screen keyboard. No title, no decorations, no place among
the windows — anchored to screen edges, in one of four depth bands, optionally
reserving space that maximised windows then keep clear of.

```rust
use obscur_kit::App;
use obscur_kit::windowing::layer::LayerSettings;

App::new("status bar")
    .app_id("com.example.bar")
    .layer(LayerSettings::top_bar(36))
    .stylesheet(CSS)
    .view(|| bar(&state.borrow()))
    .run()?
```

**That method is the whole of the difference.** Everything else is the `App` you
already know — the same `View`, the same CSS, the same commands, virtual lists,
row clicks, touch and gesture handling, the same runtime underneath. The
application's `app_id` becomes the surface's identity to the desktop; there is
no second name to set.

One method does not carry over: `view_with` is a windowed root, and using it
with `layer` returns `AppError::FrameAwareLayerRoot`. A desktop surface is sized
by the platform *before* its first frame, so there is nothing a frame-aware
builder would learn that `view` does not already have.

### The settings

`obscur_kit::windowing::layer::LayerSettings` — platform-neutral by
construction. No protocol vocabulary, no wire encoding, no sentinel values.

| | |
|---|---|
| `top_bar(h)` / `bottom_bar(h)` / `left_bar(w)` / `right_bar(w)` | a bar reserving exactly its own thickness |
| `overlay()` | whole-screen, above everything, reserving nothing and ignoring what others reserved |
| `new()` | a floating surface in the panel band, anchored to nothing |
| `with_band(LayerBand)` | `Background` · `Bottom` · `Top` · `Overlay` |
| `with_anchor(LayerAnchor)` | anchoring both edges of an axis **stretches**, and overrides the size on it |
| `with_reservation(LayerReservation)` | `None` · `Reserve(px)` · `Ignore` |
| `with_margin(LayerMargins)` · `with_focus(LayerFocusPolicy)` · `with_display(DisplayId)` | |
| `with_pointer(LayerPointerPolicy)` | `Interactive` (the default) or `Transparent` — a surface that only *shows* something lets clicks through to whatever is behind it |
| `with_size(w, h)` | an axis you leave unset is the platform's to choose — `None`, never a zero |

A display is chosen by `DisplayId` — the same id
[`WindowBackend::displays`](#embedding-a-uiruntime) hands out — rather than by a
connector name, because a name that identifies a monitor on one platform
identifies nothing on the next.

### What the platform can actually do

Desktop platforms diverge here more than they do over windows, so the port says
so rather than accepting a setting and dropping it.
`obscur_windowing::port::LayerBackend::supports` answers a `SupportLevel` for
each `LayerFeature`: `Bands`, `Reservation`, `FreeAnchor`, `ExclusiveKeyboard`,
`DisplaySelection`, `NegativeMargin`, `FractionalScale`, `InputTransparency`.

The one that most often comes back short is `Reservation` — taking space from
the desktop is an ordinary request on some platforms and has no public API at
all on others.

### Reconfiguring a running surface

The loop owns the surface and your handlers run inside it, so they cannot hold a
reference to it. `App::layer_control()` is how they reach it: a handler leaves a
request, the loop applies it at a defined point in the frame.

```rust
let app = App::new("bar").app_id("com.example.bar").layer(LayerSettings::top_bar(36));
let control = app.layer_control();
app.on_command(FLOAT, move || control.reconfigure(floating_settings()))
    .run()?
```

It also reports the surface's real `size()`, `scale()` and `state()`
(`Configuring` → `Ready` → `Closed`), and `close()` takes it down.

**No placement change needs a new surface.** Band, anchor, reservation, margin
and size are all live requests, which is why there is no "recreate". The one
exception is the display: an output is chosen when the surface is created, so
moving it comes back `conditional()` rather than `native()`.

### Waking the loop from outside

The loop sleeps until the platform has something to say. Nothing else makes it
turn — not a worker thread, not a timer. `App::waker(…)` takes the handle that
does, and it is `Send`:

```rust
let waker = LayerWaker::new()?;
let ticker = waker.clone();
std::thread::spawn(move || loop {
    std::thread::sleep(Duration::from_secs(1));
    update_shared_state();
    ticker.wake();          // the loop turns and rebuilds the view
});

App::new("bar").layer(LayerSettings::top_bar(36)).waker(waker).view(bar).run()?
```

A wake means "reconsider the interface" and nothing finer — the waker does not
know why it was rung. A worker that needs to say *what* changed puts that in the
state the view reads, where the view will actually see it.

### Where it runs

Today one backend implements the port: `wlr-layer-shell`, which wlroots
compositors (Sway, Hyprland, river, Wayfire) and KDE implement. **GNOME has
declined the protocol**, and there `run` returns `AppError::LayerUnsupported` —
a capability answer to branch on, not a crash. The fallback is an ordinary
window, reached by not calling `layer`.

The settings and the port are written to outlive that backend: nothing in
either names Wayland, and an appbar or an `NSWindow`-level backend would
implement the same `LayerBackend` and answer the same `LayerFeature` roster.

### Scale

Fractional scaling is used where the compositor offers `wp_fractional_scale_v1`
and `wp_viewporter`: the buffer is rendered at the true scale (1.6, not 2) and a
viewport maps it onto the logical size. Without both, the integer path applies
and a fractional output rounds up — which is what
`LayerFeature::FractionalScale` answers `Emulated` for.

---

## Embedding a `UiRuntime`

`UiRuntime` is the whole toolkit minus the window. It has no display, no event
loop and no clock — it takes time and input as data. A host builds one, installs
the ports it wants, and calls a frame method.

```rust
use obscur_kit::geometry::Size;
use obscur_kit::identity::RuntimeId;
use obscur_kit::runtime::UiRuntime;

let mut rt = UiRuntime::new(RuntimeId::from_raw(1), Size::new(800.0, 600.0));
let result = rt.render_now(&view, &sheets);
let snapshot = result.snapshot();      // hand to a renderer
```

The minimum useful host is four calls: `new`, install a text stack (otherwise
text measures but paints no glyphs), `render_now`, and hand the snapshot to a
renderer. [`KitBootstrap`](10-hosting.md) does the installing.

Full method list: [03 — The runtime](03-runtime.md).

## Headless

`obscur_kit::headless::HeadlessHost` drives a real `UiRuntime` with no display —
the same frame path a window takes, which is what makes it usable as an oracle
rather than as a mock. `KitBootstrap::headless_host()` builds one.

Used by the spec test suite. See [11 — Diagnostics](11-diagnostics.md) for the
record/replay tier that sits beside it (**unwired** — no shipped host installs it).

## Choosing

Take `App` unless you need to own the window. The kit's own examples do both:
`hello_app.rs`, `file_manager.rs` and `calculator.rs` are `App`; `uikit_window.rs`,
`showcase_window.rs` and `controls_window.rs` spell out their own event loop and
are the reference for embedding.
