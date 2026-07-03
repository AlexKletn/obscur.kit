# Obscur Kit 2 — Draft Specification Set

> Status: architecture draft  
> Language: Rust-first, platform-independent UI toolkit  
> Primary goals: deterministic retained UI, modern layout and styling, separate motion engine, embeddable runtime, first-class diagnostics, extensible plugin system.

## Document map

1. [Architecture and boundaries](01-architecture-and-boundaries.md)
2. [Components, tree and DSL](02-components-tree-and-dsl.md)
3. [State, transactions and runtime](03-state-transactions-and-runtime.md)
4. [Layout, CSS and motion](04-layout-css-and-motion.md)
5. [Rendering, text and resources](05-rendering-text-and-resources.md)
6. [Platform, windows and system services](06-platform-windows-and-services.md)
7. [Accessibility, testing and diagnostics](07-accessibility-testing-and-diagnostics.md)
8. [Plugin system](08-plugin-system.md)
9. [Roadmap, exclusions and unresolved details](09-roadmap-and-non-goals.md)

## Product definition

Obscur Kit 2 is a Rust-native UI toolkit with:

- a declarative external model and retained internal tree;
- CSS as the primary styling language;
- a modern layout engine inspired by browsers but without legacy web baggage;
- a standalone motion engine into which CSS transitions and keyframes are compiled;
- an independent scene/paint model and replaceable renderer backends;
- an embeddable `UiRuntime` that does not own the application process;
- headless execution, deterministic testing and external DevTools;
- typed platform capabilities and explicit fallbacks;
- optional, separately compiled plugins able to add UI modules to already-built applications.

## Core principles

1. **Explicit domain boundaries.** Data crossing subsystem boundaries uses typed domain structures. Generic maps, untyped JSON and adapter-specific data do not leak into the core model.
2. **Rust-native API.** The public API is designed for Rust rather than copying browser APIs.
3. **Friendly DX.** Internal builders, transport types and engine plumbing stay hidden behind compact declarative APIs.
4. **Platform independence.** Concrete platform types are confined to adapters.
5. **Minimal blocking.** I/O, loading, decoding and platform calls are asynchronous. Synchronous work is short, deterministic and frame-bounded.
6. **No implicit ownership of the app.** The kit manages UI runtime and windows, but not process lifetime or application policy.
7. **Recoverable capability gaps.** Unsupported platform operations return typed results instead of panicking.
8. **Diagnostics as architecture.** Errors, performance, inspection and headless testing are built in from the start.
9. **Extensions through contracts.** Plugins and external tools integrate through versioned contracts, not internal memory or renderer access.
10. **UI is not application logic.** Components may own local view state, but domain/application logic remains outside the UI layer.

## Terminology

- **Desired tree** — declarative output produced by components.
- **Retained tree** — stable internal tree used by layout, input, paint and accessibility.
- **Component** — lifecycle-bearing view unit and default error boundary.
- **Node** — a lower-level retained UI element; not automatically an error boundary.
- **UI IR** — typed, versioned intermediate representation produced by Rust DSL or `.okui`.
- **Contribution** — plugin-provided extension registered into an application extension point.
- **Host** — an application embedding Obscur Kit and optionally loading plugins.
- **Presentation value** — final value produced after style and motion resolution, before capability fallback and rendering.
