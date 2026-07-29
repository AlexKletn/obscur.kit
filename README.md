# Obscur Kit 2

Rust-native, platform-independent retained-mode UI toolkit.

It runs: an application opens a window, styles itself with real CSS, lays out
flow / flex / grid, shapes and paints text through HarfRust, and renders on the
GPU through Vello — or on the CPU, with no display, into a PNG.

- [`docs/api/`](docs/api/) — **the API documentation**: what the kit offers, in
  the order an application meets it, with an explicit line between what is wired
  end-to-end and what merely compiles.
- `cargo doc --no-deps -p obscur-kit --features full` — the exhaustive per-item
  reference. Every public item is documented; `missing_docs` is clean across the
  workspace with all features on.
- [`specs/`](specs/) — the design the implementation is measured against.
- [`docs/conformance-audit.md`](docs/conformance-audit.md) — a spec-by-spec audit
  (dated; read it with its own caveat about decay).

## Layout (spec §2.1)

```
crates/
  foundation/   (5)  identity, geometry, capability, task, diagnostics
  domain/      (15)  view, tree, interaction, data-transfer, style, layout,
                     text, motion, resources, scene, render, accessibility,
                     windowing, system, routing
  application/  (3)  runtime, headless, testkit
  frontend/     (5)  ui-ir, css, okui-syntax, okui-compiler, ui-macros
  components/   (3)  primitives, widgets, theme-system
  adapters/    (12)  window-winit, render-vello, render-cpu,
                     accessibility-accesskit, task-smol, task-tokio,
                     resource-fs, cache-fs, system-{linux,windows,macos,web}
  diagnostics/  (2)  devtools-protocol, devtools-server
  plugin/       (9)  plugin-{model,api,abi,package,host,sdk,macros,
                     executor-wasm,executor-process}
  facade/       (1)  obscur-kit
tools/          (4)  xtask (architecture checks), devtools, okui-lsp, plugin-cli
```

Source layout follows the spec's rules: one main entity per file; `mod.rs` and
`lib.rs` hold only declarations and re-exports (spec §3.1).

## Commands

```
cargo run -p xtask -- check      # architecture rules hold (mod.rs/lib.rs
                                 # layout, banned file names, unsafe boundary,
                                 # pure-crate dependency direction)
cargo run -p xtask -- consumer   # checks/facade-consumer builds: a crate whose
                                 # only dependency is the facade, which is the
                                 # only place an emitted-path defect shows up
cargo build --workspace
cargo test --workspace           # the suite is the oracle
cargo fmt --all --check
cargo clippy --workspace --all-targets
```

## Status

Working, and uneven — which is stated here rather than left to be discovered.

**Real end-to-end:** the frame pipeline (component → desired tree → retained tree
→ style → layout → scene → paint), the CSS cascade over 119 longhands, flow /
flex / grid layout, the text stack with system-font fallback, pointer and
**multi-touch** input with pan / fling / long-press, virtual lists the runtime
windows, CSS motion, `env()` safe-area insets, and both render backends.

**Not wired:** several crates compile, pass their tests, and are called by
nothing — routing, the windowing domain types, the testkit, the plugin tier. A
type existing here is not evidence that anything invokes it. The list is
[`docs/api/12-reality.md`](docs/api/12-reality.md), with the grep that produced
each entry.

The developed platform is **Linux** (X11 and Wayland, including a real tablet
target); macOS and Windows have honest placeholder providers.