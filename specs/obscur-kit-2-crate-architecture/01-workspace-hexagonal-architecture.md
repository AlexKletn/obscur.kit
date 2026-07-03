# 1. Гексагональная архитектура workspace

## 1.1 Почему она подходит

Obscur Kit 2 содержит чистые UI-домены и множество заменяемых внешних механизмов:

- window/event loop;
- renderer и GPU;
- accessibility bridge;
- filesystem/cache;
- dialogs, documents, shell и notifications;
- async executor и clock;
- headless host;
- WASM/process plugin executor;
- diagnostics transport.

Гексагональная архитектура позволяет держать правила UI независимо от ОС и backend, тестировать домены без окна и GPU и подключать внешние реализации без утечки их типов в core.

## 1.2 Архитектурные кольца

### Foundation

Минимальные стабильные типы, не знающие об UI-подсистемах:

- identity и generation;
- geometry и coordinate spaces;
- capabilities и permissions;
- task/time/cancellation abstractions;
- diagnostics model.

Foundation-крейты зависят только от других узких foundation-крейтов и небольших platform-neutral библиотек.

### Domain

Чистые bounded contexts:

- View/Component;
- retained tree;
- interaction;
- data transfer;
- style;
- layout;
- text;
- motion;
- resources;
- scene/paint;
- renderer contract;
- accessibility;
- windowing;
- system services;
- routing.

Домен может определить порт, необходимый его use cases, но не зависит от реализации этого порта.

### Application/runtime

Application layer оркестрирует домены:

- frame pipeline;
- scheduler;
- UI transactions;
- invalidation routing;
- lifecycle;
- async completion routing;
- recovery;
- adaptive-quality policy.

`obscur-runtime` зависит от доменных контрактов, но не от Winit, Vello, AccessKit, Wasmtime или конкретного executor.

### Ports

Порт находится рядом с потребителем:

```text
obscur-render
  └── port/renderer.rs

obscur-windowing
  └── port/window_backend.rs

obscur-resources
  ├── port/resource_loader.rs
  └── port/cache_store.rs

obscur-task
  ├── port/spawner.rs
  ├── port/clock.rs
  └── port/timer.rs
```

Не создаётся один огромный `PlatformApi`, объединяющий окна, renderer, clipboard, сеть и filesystem.

### Adapters

Конкретные реализации:

- Winit;
- Vello/wgpu;
- CPU renderer;
- AccessKit;
- filesystem resource/cache;
- Smol/Tokio executor;
- Linux portals и native system providers;
- Wasmtime;
- process IPC.

Adapter зависит от порта. Порт и домен никогда не зависят от adapter.

### Frontends, facades и tools

- Rust proc-macro DSL;
- `.okui` syntax/compiler;
- CSS frontend;
- umbrella crate;
- DevTools;
- plugin SDK/CLI.

Frontend переводит внешний синтаксис в доменную модель или UI IR, но не владеет runtime.

## 1.3 Направление зависимостей

Допустимо:

```text
adapter → port/domain → foundation
frontend → domain/foundation
runtime → domain/ports/foundation
facade → runtime/frontends/default adapters
tool → stable protocol/model
```

Запрещено:

```text
domain → adapter
foundation → domain
renderer adapter → runtime internals
plugin model → plugin host
UI IR → concrete widget implementation
internal crate → umbrella facade
```

## 1.4 Владение ключевыми моделями

| Модель | Владелец |
|---|---|
| typed/generational IDs | `obscur-identity` |
| logical/physical geometry | `obscur-geometry` |
| support/capability/permission | `obscur-capability` |
| structured diagnostics | `obscur-diagnostics` |
| component contract и desired view | `obscur-view` |
| retained tree и reconciliation | `obscur-tree` |
| event/focus/pointer/command | `obscur-interaction` |
| computed style и cascade | `obscur-style` |
| CSS syntax/compiler | `obscur-css` |
| measure/arrange и layout graph | `obscur-layout` |
| shaping/editing/IME | `obscur-text` |
| timelines/tracks/interpolation | `obscur-motion` |
| backend-independent Scene/Paint | `obscur-scene` |
| renderer port | `obscur-render` |
| Resource Hub и resource ports | `obscur-resources` |
| semantic accessibility tree | `obscur-accessibility` |
| window/display model и port | `obscur-windowing` |
| documents/dialogs/shell/notifications | `obscur-system` |
| frame pipeline и UI transactions | `obscur-runtime` |
| plugin DTO/contracts | `obscur-plugin-model` |
| plugin orchestration | `obscur-plugin-host` |

## 1.5 Retained tree как источник идентичности

Layout, renderer, accessibility и plugin adapters не создают конкурирующую UI-идентичность.

Все projections привязаны к:

- `NodeId`;
- generation/version;
- immutable или transactional snapshots.

Backend handles хранятся в secondary maps и никогда не заменяют доменный `NodeId`.

## 1.6 Snapshot boundaries

Между runtime и тяжёлыми подсистемами передаются versioned snapshots:

- style snapshot;
- layout input/output;
- text layout;
- scene snapshot;
- accessibility snapshot;
- render submission;
- plugin contribution update.

Snapshot после публикации неизменяем, не содержит platform-specific handles без явного extension wrapper и может быть отброшен как stale.

## 1.7 Errors и diagnostics

Публичные library APIs используют typed errors.

`anyhow` допустим только:

- в CLI и binaries;
- на верхней границе adapter;
- при немедленном преобразовании в typed diagnostic.

Diagnostics не сводится к строковым логам. Доменная модель содержит stable code, severity, subsystem, context и recovery action.

## 1.8 Unsafe policy

### Pure crates

```rust
#![forbid(unsafe_code)]
```

### Крейты, где unsafe допустим

- native plugin ABI;
- platform adapters;
- renderer adapters;
- low-level IPC/shared memory;
- FFI.

Требования:

- `#![deny(unsafe_op_in_unsafe_fn)]`;
- локальный минимальный `unsafe`-модуль;
- `SAFETY.md` в крейте;
- safety invariant у каждого unsafe block;
- Miri/sanitizer tests, где применимо;
- unsafe-типы не пересекают чистые доменные границы.

## 1.9 Когда создавать отдельный крейт

Отдельный крейт оправдан, если ответ «да» хотя бы на один вопрос:

- Это самостоятельный bounded context?
- Нужно исключать тяжёлую зависимость feature-флагом?
- Это adapter к внешней системе?
- Это стабильный API/protocol/ABI contract?
- У него отдельный target или MSRV?
- Он должен публиковаться независимо?
- Он должен тестироваться независимо от runtime?

Если нет — это модуль существующего крейта.

## 1.10 Запрет dumping-ground crates

Не создавать:

- `obscur-common`;
- `obscur-utils`;
- `obscur-shared`;
- `obscur-models`;
- `obscur-types`;
- `obscur-helpers`.

Сущность принадлежит bounded context, определяющему её семантику. По-настоящему фундаментальная семантика получает узкий leaf crate: geometry, identity, capability и т. п.

## 1.11 Правило стабильности внешних типов

Типы внешних библиотек не становятся stable public API Obscur Kit без осознанного решения.

В частности, скрываются за adapters/converters:

- `winit::*`;
- `wgpu::*`;
- `vello::*`;
- `taffy::*`;
- `parley::*`;
- `accesskit::*`;
- `wasmtime::*`.

Это позволяет обновлять или заменять implementation stack без разрушения доменной архитектуры.
