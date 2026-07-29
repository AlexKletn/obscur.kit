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
| `full` | the above plus `task-tokio` `render-vello` `window-winit` `shell-layer` `accesskit` |

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
