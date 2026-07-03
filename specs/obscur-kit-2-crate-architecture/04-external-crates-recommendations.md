# 4. Рекомендации по внешним крейтам

> Проверено по официальным репозиториям и документации на 2026-07-03.  
> Точные версии должны централизованно задаваться в `[workspace.dependencies]`; архитектурный документ фиксирует роль и ограничения интеграции, а не вечный pin.

## 4.1 Критерии отбора

Кандидат должен удовлетворять большинству условий:

- активная разработка или зрелая поддержка;
- недавние releases/commits;
- понятная maintainer/ownership model;
- совместимая лицензия;
- возможность скрыть за собственным domain/port API;
- поддержка целевых platforms;
- отсутствие известных непатченных advisories в выбранной версии.

Главное правило: внешний крейт предоставляет implementation, но не определяет архитектуру и stable public model Obscur Kit.

## 4.2 Windowing и GPU

### `winit` — рекомендуется

Роль:

- native window/event-loop adapter;
- keyboard/pointer/window events;
- display integration.

Ограничения:

- типы Winit не выходят из `obscur-window-winit`;
- `UiRuntime` не требует Winit event loop;
- embedded/overlay host может использовать другой adapter;
- platform-specific capabilities не маскируются под полностью универсальные.

Winit активно поддерживается и позиционируется как low-level window/input building block.

### `raw-window-handle` — рекомендуется на adapter boundary

Роль:

- interop между window adapter и renderer;
- создание wgpu/Vello surfaces.

Не использовать как доменную window identity.

### `wgpu` — рекомендуется внутри renderer adapters

Роль:

- cross-platform GPU abstraction;
- native и WebGPU/WASM rendering;
- trusted custom render extensions.

Не должен попадать в:

- `obscur-scene`;
- `obscur-layout`;
- component API;
- plugin UI IR.

## 4.3 Rendering stack

### `vello` — основной renderer backend

Сильные стороны:

- активная Linebender ecosystem;
- GPU compute-oriented 2D rendering;
- wgpu integration;
- близкая к требованиям vector scene модель.

Интеграция:

- только `obscur-render-vello`;
- `vello::Scene` не является public Scene Obscur Kit;
- explicit converters Scene/Paint → Vello;
- adapter принимает возможные API changes Vello/wgpu;
- backend capabilities и fallback сообщаются доменно.

### `vello_cpu` — кандидат для software/headless renderer

Используется за `obscur-render::Renderer`. Не считать его API стабильным контрактом Obscur Kit.

### `peniko`, `kurbo`, `color` — рекомендуются как internal stack

- `kurbo`: curves/path algorithms;
- `peniko`: brushes, fills, strokes, composition primitives;
- Linebender `color`: color spaces и CSS Color-oriented conversions.

Предпочтительно создать собственные stable wrappers и converters, не делать version-specific внешние типы частью stable API.

## 4.4 Layout

### `taffy` — рекомендуется как algorithm provider

Taffy реализует CSS Block, Flexbox и Grid и активно развивается.

Правильная интеграция:

- retained tree принадлежит `obscur-tree`;
- computed layout style принадлежит `obscur-layout`;
- Taffy получает tree view/children/measure callbacks;
- результат переводится в `LayoutSnapshot`;
- incremental invalidation контролирует Obscur Kit;
- constraint layout, docking и virtualization остаются собственными algorithms.

Не раскрывать `taffy::Style` и `taffy::NodeId` в public API.

## 4.5 CSS

### `cssparser` — рекомендуется

Использовать как низкоуровневый parser CSS Syntax.

Он не должен определять semantic property model: property parsing/compiler остаётся в `obscur-css`, cascade/computed style — в `obscur-style`.

### `selectors` из Servo/Stylo — рекомендуется

Использовать для selector parsing/matching против собственного tree adapter.

Следует ориентироваться на актуальный Servo Stylo repository, а не старый перенесённый standalone repository.

### `lightningcss` — опционально

Полезен для build-time:

- transformation;
- minification;
- normalization;
- packaging pipeline.

Не рекомендуется как владелец runtime cascade/computed-style semantics.

### Полный Stylo — пока не рекомендуется

Причины:

- browser-scale integration;
- HTML/browser-specific assumptions;
- сложный lifecycle и threading model;
- затруднение собственного capability/style contract.

Stylo полезен как reference implementation и источник tests/algorithms.

## 4.6 Text и fonts

### `parley` — основной кандидат text layout engine

Роль:

- rich text layout;
- shaping orchestration;
- line breaking;
- bidi;
- selection/editing geometry.

Интеграция:

- закрыть за `TextShaper`/`TextLayoutEngine`;
- `TextLayout`, `GlyphRun`, `TextSelection` остаются доменными;
- Parley types не передаются widgets или plugins;
- добавить conformance tests для scripts, bidi, ligatures и fallback.

### `fontique` — рекомендуется через Linebender stack

Использовать для font enumeration/fallback там, где поддерживается.

### `harfrust` и `skrifa` — рекомендуются как low-level stack

- HarfRust: shaping;
- Skrifa/fontations: OpenType parsing, metrics, outlines.

Предпочтительно сначала зависеть транзитивно через Parley. При прямом использовании — собственные adapters и conformance corpus.

HarfRust активно развивается, но не следует предполагать полную эквивалентность всех возможностей HarfBuzz без тестирования.

## 4.7 Accessibility

### `accesskit` — рекомендуется

Использовать через `obscur-accessibility-accesskit`.

Semantic tree Obscur Kit остаётся собственной моделью; AccessKit получает преобразованные snapshots/actions.

При необходимости меньшей связанности с Winit рассмотреть актуальные cross-platform adapters AccessKit.

## 4.8 Async, tasks и concurrency

### `futures-core` / `futures-util` — рекомендуется

Использовать стандартные `Future`/`Stream` abstractions без привязки к executor.

Не включать полный feature set, когда достаточно `futures-core`.

### `async-task` — кандидат для default executor adapter

Подходит для лёгкого собственного scheduler/executor adapter.

Не нужен в domain API.

### `async-channel` — рекомендуется

Использовать bounded channels для backpressure.

### `crossbeam` — рекомендуется для native worker communication

Use cases:

- bounded queues;
- work stealing;
- synchronization;
- renderer/resource worker communication.

Предпочитать узкие subcrates, когда не нужен весь umbrella.

### `rayon` — только для явных CPU jobs

Кандидаты:

- image decode batches;
- stylesheet compilation;
- expensive scene preparation;
- font preprocessing;
- offline export.

Правила:

- собственный bounded pool;
- не использовать global pool без policy;
- не выполнять UI mutations;
- учитывать power profile и plugin/resource quotas.

### Tokio — не core dependency

Допустим только как:

- `obscur-task-tokio`;
- process/IPC adapter;
- network provider;
- application host choice.

## 4.9 Data structures

### `slotmap` — рекомендуется

Подходит для:

- generational retained-node storage;
- secondary maps;
- stable internal handles.

Public API использует собственные typed IDs.

### `smallvec` — после profiling

Возможные hot paths:

- short child lists;
- event paths;
- hit chains;
- clip stacks;
- small property collections.

Не применять автоматически ко всем `Vec`.

### `hashbrown` — при необходимости контроля

Для custom hasher/allocator и high-performance maps. Обычный `std::collections::HashMap` остаётся нормальным default.

### `indexmap` — где порядок семантически важен

- source-order declarations;
- deterministic registries;
- manifest sections;
- stable diagnostics output.

### `bitflags` — рекомендуется для независимых flags

Не использовать, если допустимые комбинации должны выражаться state-machine enum.

## 4.10 Errors и instrumentation

### `thiserror` — рекомендуется

Для typed library errors.

### `tracing` — рекомендуется как instrumentation adapter

Роль:

- spans;
- performance/debug instrumentation;
- adapter logs.

Stable diagnostics остаются в `obscur-diagnostics` и при необходимости зеркалируются в `tracing`.

`anyhow` — только binaries/CLI/top adapter boundary.

## 4.11 Serialization и schemas

### `serde` — рекомендуется

Use cases:

- manifests;
- settings;
- diagnostics DTO;
- configs;
- fixtures.

Не derive serialization для всех hot runtime entities автоматически.

### `schemars` — рекомендуется

Для:

- plugin manifest JSON Schema;
- settings schema;
- generated docs;
- debug/protocol schema inspection.

### `postcard` — кандидат compact message codec

Подходит для local IPC и bounded internal messages, если поверх него есть:

- explicit envelope version;
- stable numeric tags;
- compatibility tests;
- payload limits;
- unknown-field/variant policy.

Postcard сам по себе не решает schema evolution.

### `rkyv` — не основной долговечный plugin wire protocol

Допустим для:

- same-version internal snapshots;
- validated cache artifacts;
- performance-critical trusted data.

Не рекомендуется для:

- untrusted plugin package input без строгой validation;
- долгоживущего mixed-version protocol;
- C ABI;
- основного UI IR contract.

Zero-copy representation слишком тесно связывает wire layout с Rust model и усложняет evolution.

## 4.12 Proc macros и language tooling

### `syn`, `quote`, `proc-macro2` — рекомендуется

Для Rust TSX-like DSL и derives. Включать только необходимые Syn features для контроля compile time.

### `logos` — рекомендуется для `.okui` lexer

### `rowan` — рекомендуется для lossless syntax tree

Полезен parser/formatter/LSP и source-preserving edits.

### `salsa` — оценить позже

Подходит для incremental compiler/LSP queries, но не обязателен для первой версии compiler. Внедрять после появления реальной incremental workload и измерений.

## 4.13 Plugin execution

### `wasmtime` — основной кандидат WASM executor

Преимущества:

- активная Bytecode Alliance development;
- embedding API;
- Cranelift;
- WASI/component ecosystem;
- fuel/epoch interruption и resource control mechanisms.

Требования:

- скрыть за `PluginExecutor`;
- не пропускать Wasmtime types в Plugin API;
- отслеживать security patch releases;
- задавать memory/fuel/epoch/resource limits;
- не выдавать WASI filesystem/network без host capability policy.

Component Model/WIT можно изучить для generated bindings, но не делать обязательной основой native/process transports до proof of concept.

### Process IPC

Transport не фиксируется в semantic Plugin API.

Кандидаты:

- Unix domain sockets;
- Windows named pipes;
- local TCP только при необходимости;
- platform message channels;
- shared memory только для отдельных high-throughput trusted extensions.

Первая версия: framed messages + bounded queues, без shared memory.

## 4.14 Packages и compression

### `zip` из `zip-rs/zip2` — рекомендуется

Использовать streaming entry reader и собственную validation:

- normalized path;
- file count;
- compressed/uncompressed limits;
- compression-ratio limits;
- no partial activation.

Включать только реально нужные codecs/features.

### `tar` — только patched version

Минимальный security floor: **`tar >= 0.4.45`**.

Даже на patched версии:

- не использовать blind `unpack` для plugin packages;
- перечислять entries;
- отвергать symlink/hardlink по умолчанию;
- валидировать normalized path;
- применять собственные quotas;
- извлекать native entrypoint только по manifest.

### `flate2`, `zstd` — рекомендуется

Для `tar.gz` и `tar.zst`. Compression backend выбирается осознанно через features.

### `blake3` — рекомендуется

Для:

- content-addressed cache;
- package hashes;
- resource fingerprints;
- incremental compiler keys.

BLAKE3 hash не является publisher signature.

## 4.15 Testing

### `proptest` — рекомендуется

- geometry transforms;
- layout invariants;
- reconciliation;
- parser properties;
- codecs;
- archive path validation.

### `insta` — рекомендуется

- syntax trees;
- UI IR;
- computed styles;
- layout snapshots;
- diagnostics;
- Scene/Paint snapshots.

Raster comparison требует отдельного tolerance-aware helper.

### `trybuild` — рекомендуется

- proc-macro compile-pass/fail;
- DSL diagnostics;
- plugin SDK derives.

### `cargo-nextest` — рекомендуется

Основной CI test runner для большого workspace.

### `criterion` — рекомендуется

Benchmarks:

- reconciliation;
- selectors/cascade;
- layout;
- shaping;
- scene building;
- plugin codecs/executors.

### `loom` — точечно

Для собственных concurrency primitives, cancellation, handle invalidation и queues.

### Fuzzing

`cargo-fuzz`/libFuzzer targets:

- CSS;
- `.okui`;
- UI IR;
- plugin manifest;
- ZIP/TAR entry validation;
- own SVG/font/image boundaries.

## 4.16 Итоговая таблица

| Область | Предпочтительно | Роль |
|---|---|---|
| Windowing | `winit`, `raw-window-handle` | adapter only |
| GPU | `wgpu` | renderer adapter |
| 2D render | `vello` | primary backend |
| CPU render | `vello_cpu` | optional backend |
| Vector stack | `kurbo`, `peniko`, `color` | internal/adapter |
| Layout | `taffy` | algorithm provider |
| CSS | `cssparser`, `selectors` | parser/matcher |
| Text | `parley`, `fontique`, `harfrust`, `skrifa` | internal engine |
| Accessibility | `accesskit` | adapter |
| Async | `futures-core`, `async-task`, `async-channel` | ports/adapters |
| Concurrency | `crossbeam`, `rayon` | bounded workers |
| IDs/storage | `slotmap` | internal storage |
| Errors/telemetry | `thiserror`, `tracing` | errors/instrumentation |
| Schemas | `serde`, `schemars` | manifests/settings |
| Compact codec | `postcard` | evaluated wire codec |
| WASM plugins | `wasmtime` | executor adapter |
| Archives | `zip`, `tar >= 0.4.45`, `flate2`, `zstd` | plugin packages |
| Hashing | `blake3` | content fingerprints |
| DSL tooling | `syn`, `quote`, `proc-macro2`, `logos`, `rowan` | frontends |
| Tests | `proptest`, `insta`, `trybuild`, `nextest` | quality |

## 4.17 Официальные источники

- Winit: <https://github.com/rust-windowing/winit>
- raw-window-handle: <https://github.com/rust-windowing/raw-window-handle>
- wgpu: <https://github.com/gfx-rs/wgpu>
- Vello: <https://github.com/linebender/vello>
- Parley: <https://github.com/linebender/parley>
- Linebender projects: <https://linebender.org/>
- Taffy: <https://github.com/DioxusLabs/taffy>
- cssparser: <https://github.com/servo/rust-cssparser>
- Stylo/selectors: <https://github.com/servo/stylo>
- AccessKit: <https://github.com/AccessKit/accesskit>
- Wasmtime: <https://github.com/bytecodealliance/wasmtime>
- Wasmtime release policy: <https://docs.wasmtime.dev/stability-release.html>
- tracing: <https://github.com/tokio-rs/tracing>
- Serde: <https://github.com/serde-rs/serde>
- Schemars: <https://github.com/GREsau/schemars>
- Postcard: <https://github.com/jamesmunns/postcard>
- rkyv: <https://github.com/rkyv/rkyv>
- zip-rs: <https://github.com/zip-rs/zip2>
- tar-rs: <https://github.com/composefs/tar-rs>
- tar advisory RUSTSEC-2026-0067: <https://rustsec.org/advisories/RUSTSEC-2026-0067.html>
- tar advisory RUSTSEC-2026-0068: <https://rustsec.org/advisories/RUSTSEC-2026-0068.html>
- BLAKE3: <https://github.com/BLAKE3-team/BLAKE3>
- Rowan: <https://github.com/rust-analyzer/rowan>
- Logos: <https://github.com/maciejhirsz/logos>
- Salsa: <https://github.com/salsa-rs/salsa>
