# Obscur Kit 2

Rust-native, platform-independent retained-mode UI toolkit.

This repository contains the **full target workspace skeleton**: every crate
from the crate-architecture spec (§2.1) and the per-crate source layout (§3),
materialised as compiling stubs, plus the Phase 0 enforcement layer (workspace,
lints, architecture `xtask`).

See [`specs/`](specs/) for the full design.

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
cargo build --workspace          # compiles (stubs emit expected warnings)
cargo fmt --all --check
cargo clippy --workspace --all-targets
```

## Status

Structure-complete, behaviour-empty. Most entity files are placeholder stubs
(`pub struct <Name>;`) that materialise the target layout; foundation crates
(`obscur-identity`, `-geometry`, `-capability`, `-task`, `-diagnostics`) and the
first vertical-slice types carry real starter content. The Phase 1 pipeline
`component -> desired tree -> retained tree -> style -> layout -> scene`
(spec §5.13) is the next fill-in step.
