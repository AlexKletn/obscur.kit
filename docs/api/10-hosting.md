# 10 — Hosting

`obscur-kit` is an umbrella: it contains no implementation, re-exports every
toolkit crate under a short alias, and adds three things of its own — the
`prelude`, the `App` entry, and `KitBootstrap`.

```rust
obscur_kit::scene::Color        // == obscur_scene::color::Color
obscur_kit::runtime::UiRuntime  // == obscur_runtime::runtime::UiRuntime
```

---

## Cargo features

`default` is portable: it needs no display and no GPU.

```toml
obscur-kit = "…"                                  # default
obscur-kit = { version = "…", features = ["full"] }
```

| Set | Contents |
|---|---|
| `default` | `resource-fs` `resource-decode` `cache-fs` `task-smol` `render-cpu` `text-harfrust` `text-unicode` `font-fontconfig` |
| `full` | the above plus `task-tokio` `render-vello` `window-winit` `shell-layer` `accesskit` `plugin-process` |
| `plugins` | Plugin host and package loader |
| `plugin-process` | `plugins` plus the process executor and framed transport |
| `plugin-wasm` | `plugins` plus the WASM executor |

The real text shaper is in `default` on purpose. It used to sit behind a feature
nothing turned on, so the out-of-the-box shaper was the *mock* and every host
hand-wired the real one. The adapters are pure Rust with no display, GPU or IO
reach; they do not make text appear by themselves — a host still hands over font
bytes — they make it possible to without opting in first.

`font-fontconfig` is in `default` for that reason plus one more: CI runs a plain
`cargo test --workspace`, so a feature-gated test of the system-font path would
be **silently skipped**. The feature only makes the code reachable; nothing
shells out until a host calls `system_font_set`.

`App` requires `window-winit` + `render-vello`, i.e. `full`.

## `KitBootstrap`

The batteries. Everything it installs is optional and explicit — there is no
hidden global state.

```rust
let boot = KitBootstrap::new();
boot.install_default_stylesheet(&mut rt);
boot.install_image_resources(&mut rt, ".");
boot.install_vector_resources(&mut rt);
let _spawner = boot.install_blocking_executor(&mut rt);
let installed = boot.install_text_stack(&mut rt, &boot.system_font_set().unwrap_or_default());
```

| Method | |
|---|---|
| `new()`, `with_cache_root(impl Into<PathBuf>)` | |
| `headless_host() -> HeadlessHost` | |
| `ua_stylesheet() -> Stylesheet`, `install_default_stylesheet(&mut UiRuntime)` | |
| `default_renderer() -> CpuRenderer` | |
| `default_resource_loader() -> FsResourceLoader`, `default_image_decoder() -> ImageDecoder` | |
| `install_image_resources(&mut UiRuntime, sandbox_root)` | **note the sandbox root** — `url()` resolution is confined to it |
| `install_vector_resources(&mut UiRuntime)` | SVG |
| `install_accessibility(&mut UiRuntime) -> Rc<RefCell<AccessKitBridge>>` | `accesskit` feature |
| `default_cache_store() -> Option<FsCacheStore>` | |
| `default_task_spawner() -> SmolSpawner`, `install_blocking_executor(&mut UiRuntime) -> Arc<SmolSpawner>` | keep the returned handle alive |
| `system_shell_provider() -> LinuxShellProvider` | |
| text stack | see [07](07-text-and-fonts.md) |

## Adapters

| Area | Adapter | Feature |
|---|---|---|
| Render (CPU) | `obscur-render-cpu` | `render-cpu` |
| Render (GPU) | `obscur-render-vello` | `render-vello` |
| Window | `obscur-window-winit` | `window-winit` |
| Layer shell (panels, docks, overlays) | `obscur-shell-layer` | `shell-layer` |
| Text shaping | `obscur-text-harfrust` | `text-harfrust` |
| Unicode | `obscur-text-unicode` | `text-unicode` |
| Hyphenation | `obscur-text-hyphenation` | |
| Fonts | `obscur-font-fontconfig` | `font-fontconfig` |
| Resources | `obscur-resource-fs`, `obscur-resource-decode` | `resource-fs`, `resource-decode` |
| Cache | `obscur-cache-fs` | `cache-fs` |
| Tasks | `obscur-task-smol`, `obscur-task-tokio` | `task-smol`, `task-tokio` |
| Accessibility | `obscur-accessibility-accesskit` | `accesskit` |
| System services | `obscur-system-linux` / `-web` / `-macos` / `-windows` | selected by target |

Enable `accesskit-window` to connect the built-in winit runner to the
platform accessibility adapter. The runner publishes semantic updates
when the adapter is active and dispatches its action requests through the
runtime. Custom window hosts can install and drive the bridge through
`KitBootstrap::install_accessibility` with the `accesskit` feature.

Applications can install a `CaptureProvider` on `AppControl` and request a
capture through `AppControl::capture`. The provider supplies image bytes and
reports whether it can enforce the request's protected regions. A protected
request is denied before capture when the provider lacks that capability.
`control::SurfaceCapture` is a provider for an application's rendered frame.
Supply premultiplied RGBA8 pixels using `set_frame`. Whole-surface and region
requests use physical frame pixels, and protected rectangles are rounded
outward and cleared to transparent in the captured image.

With `window-winit`, `AppControl::capture_next_frame` requests a readback
from the built-in window runner. Surface, region, current-window and node
requests share a single readback when handled in the same frame. A node
request crops the composited frame to its transformed border box. The
callback receives the image on the window thread.

`AppControl::set_capture_protected_regions` applies protection to all
captures through that control. A queued window capture keeps the regions
that were protected when requested and adds any regions protected when the
runner fulfills it.

`AppControl::set_window_capture_protected` requests whole-window system
capture protection on Windows and macOS. The support level is limited
because the windowing API leaves enforcement to the OS. The winit Wayland
and X11 backends report this request as unavailable.

## Platform support, honestly

| Target | State |
|---|---|
| **Linux** (X11 + Wayland) | the developed target. Windowing, GPU, fonts, system services |
| **wasm32** | a real provider; a host supplies faces |
| **macOS** | system provider present; **font enumeration not implemented** |
| **Windows** | same |

The v1 of this kit ran in production on Windows; v2's platform work has gone to
Linux first, and the two adapters above are honest placeholders rather than
finished ports.

### Cross-compiling

Nothing links a system library: Vulkan and Wayland are `dlopen`ed. A release
`aarch64-unknown-linux-gnu` build needs only the Rust target and a cross linker,
and the resulting binary needs `libc`, `libm` and `libgcc_s`.

```sh
rustup target add aarch64-unknown-linux-gnu
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  cargo build --release --target aarch64-unknown-linux-gnu --features full
```

## Desktop panels and local portals

Call `App::layer_capabilities()` before creating a desktop panel. With the
`shell-layer` feature the adapter checks the connected compositor for
`zwlr_layer_shell_v1` and reports feature support, including fractional
scaling, keyboard input and native popups. A Wayland connection without
the required protocol returns `LayerError::Unsupported`.

Use `App::layer(settings)` when a panel is required. With `window-winit`,
`App::layer_or_window(settings)` opts into an ordinary-window fallback when
the panel adapter or the requested panel keyboard input is unavailable.
Other discovery errors are returned. Hosts can implement `LayerPlatform`
for discovery and `LayerBackend` for panel operations.

Panels receive keyboard input with `LayerFocusPolicy::Exclusive` or
`LayerFocusPolicy::OnDemand` when the compositor grants focus. On-demand focus
requires layer-shell v4. The default `Never` leaves keyboard focus with other
surfaces. Keys, modifiers, and repeat use the regular runtime input path. The
panel adapter supports direct keyboard text input through the seat keymap.
On compositors with `text-input-v3`, writable panel fields also receive IME
preedit and committed text. Check `LayerFeature::TextInput` for availability.
The adapter sends a bounded UTF‑8 window around the selection and uses the
field's rectangle to position candidates. IME inside native popups remains
pending.

Use `LayerControl::show_popup_view(node, side, (width, height), view)`
to open a native popup anchored to a laid-out panel element. Dimensions are
logical pixels. The popup has its own `UiRuntime`, applies the panel's
stylesheets and fonts, and handles mouse, touchscreen and wheel input.
Use `show_popup_view_with` to provide a reactive view builder and configure
that runtime, for example to register popup commands. Call
`request_popup_frame` to refresh a reactive popup after application state
changes outside a popup event.

`LayerControl::show_popup(node, side, frame)` accepts a premultiplied RGBA8
`LayerPopupFrame` for application-rendered content. The compositor may flip or
slide either kind of popup to keep it on screen. `on_popup_event` delivers the
configured rectangle, popup-local input, failures, and closure.
`close_popup()` closes the surface. Each panel hosts one popup at a time.
Use `show_popup_menu` or `show_popup_menu_with` inside a native panel
pointer-down, touch-down, or key-down handler to request a Wayland grab.
The toolkit uses the initiating event's activation. The compositor can
deny the grab and dismiss the popup. A grabbed menu closes on Escape
or outside interaction. The panel restores focus to the anchor when
native focus returns. Popup views receive keys and modifiers when the
compositor focuses the popup. During an active menu grab, the adapter
also routes keys to the menu if the compositor keeps native focus on the panel.
Use a keyboard-capable panel focus policy to allow this route.

Local portals are available inside an ordinary window or a panel. Author
`<Portal target="overlay">...</Portal>` with a unique `id="overlay"` on a
host element in the same view. The portal keeps its logical ancestors for
style and command routing. Layout, painting, transforms and hit testing
use its visual host. An unresolved, ambiguous or cyclic target leaves the
portal in its original position. Use `AppControl::set_portal_host_resolver`
or the corresponding `UiRuntime` method to install a custom resolver.

## Layer discipline

The workspace is layered foundation → domain → application → adapters →
components → frontend → facade, and the boundaries are **machine-enforced**:

```sh
cargo run -p xtask -- check
```

A domain crate cannot reach an adapter; the facade may re-export anything and
implements nothing. This is the rule that keeps `UiRuntime` platform-free.

## Structural caveat

`obscur-kit/src/lib.rs` is a re-export barrel. **Anything whose only path to the
outside is that file is unwired by construction** — it is exported, it compiles,
and nothing in an application path calls it. That is the mechanism behind most of
[12 — What is not wired](12-reality.md), and it is worth knowing that a `pub use`
here is not evidence of anything but availability.
