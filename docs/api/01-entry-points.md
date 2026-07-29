# 01 — Entry points

Three ways in, in increasing order of how much you own yourself.

| | You supply | You own | Feature |
|---|---|---|---|
| [`App`](#app) | a view builder and handlers | nothing | `window-winit` + `render-vello` |
| [`UiRuntime`](#embedding-a-uiruntime) | the window and the loop | everything around a frame | any |
| [`HeadlessHost`](#headless) | nothing | a deterministic driver | any |

---

## `App`

`obscur_kit::App` — the one-call windowed entry (`src/app/app.rs`). It owns the
window, the GPU surface, the event loop, the text stack and the frame pipeline.
It exists only under `--features full` (or `window-winit` + `render-vello`) and
is absent from the headless default build.

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
| `stylesheet(&str) -> Self` | appends a sheet. Callable repeatedly; later sheets cascade over earlier ones |
| `view(impl Fn() -> View + 'static) -> Self` | the root builder, re-invoked per frame |
| `view_with(impl FnMut(&FrameInfo) -> View + 'static) -> Self` | the root builder, given the live viewport and scale |
| `screen(&str) -> Self` | the root as `.okui` markup, compiled at `run` |
| `on_command(InternedSymbol, impl FnMut() + 'static) -> Self` | a handler for a command name bound in the view or the sheet |
| `on_scroll(impl FnMut(f32) + 'static) -> Self` | wheel deltas **no `overflow: scroll` container consumed** |
| `virtual_list(InternedSymbol, impl FnMut(usize) -> ViewNode + 'static) -> Self` | the row builder for a list the runtime windows |
| `on_row_click(InternedSymbol, impl FnMut(usize, u32) + 'static) -> Self` | `(model index, click count)` for a row of that list |
| `on_row_long_press(InternedSymbol, impl FnMut(usize) + 'static) -> Self` | a held row of that list |
| `run(self) -> Result<(), AppError>` | opens the window and runs to completion |

`view`, `view_with` and `screen` set the same slot: **the last one called wins.**

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
