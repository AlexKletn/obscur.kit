# 5. Cargo-политики и порядок реализации

## 5.1 Root workspace

Базовый вариант:

```toml
[workspace]
resolver = "3"
members = [
    "crates/foundation/*",
    "crates/domain/*",
    "crates/application/*",
    "crates/frontend/*",
    "crates/components/*",
    "crates/adapters/*",
    "crates/diagnostics/*",
    "crates/plugin/*",
    "crates/facade/*",
    "tools/*",
]
default-members = [
    "crates/facade/obscur-kit",
]

[workspace.package]
edition = "2024"
rust-version = "<chosen-msrv>"
license = "MIT OR Apache-2.0"
repository = "<repository-url>"

[workspace.dependencies]
# Все версии централизуются здесь.

[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "deny"
missing_docs = "warn"
unreachable_pub = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
module_name_repetitions = "allow"
too_many_lines = "allow"
```

Каждый package наследует:

```toml
[lints]
workspace = true
```

## 5.2 MSRV

Рекомендуется один workspace MSRV для public/core crates.

Не стоит обещать слишком старый MSRV для нового UI toolkit: graphics/text ecosystem развивается быстро. Выбранный MSRV должен:

- поддерживаться CI;
- быть записан в `workspace.package.rust-version`;
- обновляться явно через changelog/decision record;
- учитывать минимальные версии Winit/Vello/Parley/AccessKit stack.

Adapter может потребовать более новый Rust только при серьёзной необходимости. В таком случае feature/default set и фактический MSRV facade документируются отдельно.

## 5.3 Feature policy

Feature flags физически исключают:

- backend;
- platform adapter;
- plugin executor;
- diagnostics server;
- media integration;
- default theme;
- heavy codecs.

Features не должны менять фундаментальную семантику core.

Пример facade:

```toml
[features]
default = [
    "window-winit",
    "render-vello",
    "accessibility-accesskit",
    "task-smol",
    "theme-system",
]

window-winit = ["dep:obscur-window-winit"]
render-vello = ["dep:obscur-render-vello"]
render-cpu = ["dep:obscur-render-cpu"]
accessibility-accesskit = ["dep:obscur-accessibility-accesskit"]
task-smol = ["dep:obscur-task-smol"]
task-tokio = ["dep:obscur-task-tokio"]
theme-system = ["dep:obscur-theme-system"]
plugins = ["dep:obscur-plugin-host"]
plugin-wasm = ["plugins", "dep:obscur-plugin-executor-wasm"]
plugin-process = ["plugins", "dep:obscur-plugin-executor-process"]
devtools = ["dep:obscur-devtools-server"]
```

Domain crates должны иметь минимум features.

## 5.4 Default features внешних зависимостей

Для тяжёлых crates:

```toml
dependency = { workspace = true, default-features = false, features = [...] }
```

Особенно внимательно:

- wgpu;
- Vello;
- Wasmtime;
- image/font codecs;
- zip/compression;
- Tokio;
- tracing subscribers;
- Serde derive.

Каждый внешний feature должен быть осознан и отражён в dependency policy.

## 5.5 Publish policy

Публичные кандидаты:

- foundation crates;
- domain contracts;
- runtime;
- UI IR;
- facade;
- plugin model/API/SDK.

Internal implementation crates могут иметь:

```toml
publish = false
```

Для публикуемых sibling dependencies:

```toml
obscur-geometry = { path = "../obscur-geometry", version = "0.x.y" }
```

Не публиковать adapter автоматически только потому, что он выделен в package.

## 5.6 Dependency rules

### Pure crates могут зависеть от

- foundation;
- явно разрешённых lower-domain crates;
- небольших platform-neutral libraries.

### Pure crates не могут зависеть от

- Winit;
- raw-window-handle;
- wgpu;
- Vello;
- AccessKit platform adapters;
- Wasmtime;
- Tokio runtime;
- native filesystem/network crates.

### Umbrella dependency запрещён

Ни один внутренний package не зависит от `obscur-kit`.

Facade является вершиной графа.

## 5.7 Lint policy

Pure domain crate:

```rust
#![forbid(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
```

Adapter/ABI crate:

```rust
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
```

Дополнительно рекомендуются:

- `clippy::unwrap_used = deny` в library code;
- `clippy::expect_used = warn` или `deny`;
- `clippy::panic = deny` в recoverable runtime paths;
- `clippy::todo = deny` в release branches;
- `clippy::dbg_macro = deny`;
- централизованный exception list.

## 5.8 Architecture xtask

```text
tools/xtask/
└── src/
    ├── main.rs
    ├── command/
    │   ├── mod.rs
    │   ├── check_architecture.rs
    │   ├── check_modules.rs
    │   └── check_dependencies.rs
    └── rule/
        ├── mod.rs
        ├── mod_rs_rule.rs
        ├── forbidden_file_name.rs
        ├── dependency_direction.rs
        └── unsafe_boundary.rs
```

Проверки:

- `mod.rs` содержит только declarations/reexports;
- `lib.rs` не содержит entity implementations;
- нет dumping-ground filenames;
- domain не зависит от adapters;
- internal packages не зависят от facade;
- unsafe разрешён только allowlist;
- platform dependency находится только в adapter/tool;
- один source file не экспортирует несколько главных сущностей без exception marker.

## 5.9 Cargo graph rules

На базе `cargo metadata`:

```text
foundation -> foundation only
domain -> foundation + approved lower domain
application -> foundation + domain + ports
frontend -> foundation + domain models
adapter -> foundation + domain port + external implementation
facade -> any selected public/default package
plugin-model -> foundation + UI IR model
plugin-host -> runtime + plugin APIs
```

CI падает при нарушении.

## 5.10 CI matrix

### Каждый pull request

- `cargo fmt --check`;
- `cargo clippy --workspace --all-targets`;
- unit tests;
- `cargo nextest run`;
- architecture checks;
- feature combinations для ключевых crates;
- proc-macro `trybuild`;
- snapshots;
- fuzz corpus smoke tests для parsers/package validation.

### Периодически

- Miri для foundation/domain;
- ASan/UBSan для adapters/ABI;
- full fuzzing;
- GPU backend matrix;
- Wayland/X11/Windows/macOS;
- WASM build;
- MSRV build;
- plugin compatibility matrix;
- dependency advisory scan (`cargo-deny`/`cargo-audit`).

## 5.11 Benchmarks

Обязательные направления:

- reconciliation;
- selector matching;
- cascade/computed style;
- style/layout invalidation;
- flow/flex/grid;
- text shaping;
- scene building;
- display-list cache;
- renderer submission;
- resource cache;
- plugin codecs;
- WASM/process calls;
- package validation.

Benchmark измеряет доменную операцию отдельно от adapter overhead.

## 5.12 Порядок реализации

### Phase 0 — enforcement

- root workspace;
- lints;
- architecture xtask;
- CI;
- identity/geometry/diagnostics;
- dependency policy.

### Phase 1 — headless vertical slice

- View;
- retained tree;
- reconciliation;
- style model;
- minimal CSS;
- flow/flex;
- mock intrinsic text measurement;
- Scene/Paint;
- headless snapshots.

Цель:

```text
component → desired tree → retained tree → style → layout → scene
```

без ОС и GPU.

### Phase 2 — first native window

- task adapter;
- Winit adapter;
- Vello adapter;
- Resource Hub;
- fonts/text;
- event-driven frame scheduler;
- first interactive window.

### Phase 3 — interaction и editing

- pointer/focus/commands;
- hit testing;
- scroll;
- editing/IME;
- DnD/clipboard;
- accessibility.

### Phase 4 — motion и advanced layout

- independent motion engine;
- CSS transition/keyframe frontend;
- grid;
- virtualization;
- docking;
- constraint layout;
- Canvas;
- offscreen.

### Phase 5 — language/tooling

- UI IR stabilization;
- `.okui` parser/compiler;
- LSP;
- DevTools protocol/server;
- record/replay;
- diagnostics snapshots.

### Phase 6 — plugins

- plugin model/API;
- extension points;
- UI IR contributions;
- package reader;
- host lifecycle/policy;
- native ABI;
- WASM executor;
- process executor;
- SDK/CLI.

## 5.13 Первый реальный skeleton

```text
crates/foundation/
├── obscur-identity/
├── obscur-geometry/
├── obscur-capability/
├── obscur-task/
└── obscur-diagnostics/

crates/domain/
├── obscur-view/
├── obscur-tree/
├── obscur-style/
├── obscur-layout/
├── obscur-resources/
├── obscur-scene/
└── obscur-render/

crates/application/
├── obscur-runtime/
└── obscur-headless/
```

До Winit/Vello integration headless tests должны доказать:

- stable identity;
- deterministic reconciliation;
- computed-style correctness;
- incremental layout;
- deterministic Scene/Paint output;
- transaction boundaries;
- structured diagnostics.

## 5.14 Решения, которые нужно оставить обратимыми

Скрыть за ports/adapters:

- глубину Taffy integration;
- версию и устройство Vello backend;
- Parley/HarfRust internals;
- default executor;
- UI IR binary codec;
- plugin message codec;
- Wasmtime или альтернативный engine;
- IPC transport;
- cache format;
- diagnostics transport.

## 5.15 Решения, которые стабилизируются рано

- typed IDs;
- coordinate spaces;
- component contract semantics;
- event propagation;
- transaction boundaries;
- cascade semantics;
- measure/arrange contract;
- Scene/Paint domain model;
- capability result model;
- diagnostic codes/context;
- UI IR version rules;
- plugin contribution/extension-point semantics.
