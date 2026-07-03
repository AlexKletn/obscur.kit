# 1. Architecture and Boundaries

## 1.1 Architectural style

Obscur Kit 2 uses a modular, layered architecture with clean domain boundaries.

Primary domains:

- Component / View
- Tree Runtime
- Interaction
- Style
- Layout
- Text
- Motion
- Scene / Paint
- Renderer
- Windowing
- Resources
- Accessibility
- Diagnostics
- Platform adapters
- Optional plugin infrastructure

Dependencies point toward abstractions. Concrete window-system, GPU, filesystem, networking and platform types never leak upward.

## 1.2 Public and internal API levels

APIs are classified as:

- **stable** — intended for normal application use;
- **extension** — supported contracts for providers and integrations;
- **experimental** — opt-in, version-unstable features;
- **internal** — not part of compatibility guarantees.

Independent version domains:

- Rust public API;
- supported CSS subset;
- UI IR;
- diagnostics protocol;
- resource manifests;
- plugin contracts;
- extension/provider contracts.

## 1.3 Crate decomposition

Indicative crates:

- `obscur-kit`
- `obscur-kit-core`
- `obscur-kit-tree`
- `obscur-kit-style`
- `obscur-kit-layout`
- `obscur-kit-text`
- `obscur-kit-motion`
- `obscur-kit-scene`
- `obscur-kit-render-api`
- `obscur-kit-render-vello`
- `obscur-kit-resources`
- `obscur-kit-windowing`
- `obscur-kit-accessibility`
- `obscur-kit-headless`
- platform adapter crates;
- optional integration crates;
- plugin API, SDK and executor crates.

Unused backends and optional integrations must not be linked.

## 1.4 Runtime topology

`UiRuntime` is embeddable.

It may run in:

- standalone mode with kit-created windows;
- embedded mode inside an existing host window, surface or event loop;
- overlay mode where another engine owns rendering and lifecycle;
- headless mode without windows or GPU.

Multiple runtimes may coexist in one process. Global singletons are avoided except where the platform makes them unavoidable.

A `ResourceHub` may be shared, but sharing is optional and explicit.

## 1.5 Threading model

The UI runtime has a single logical owner and processes UI mutations sequentially.

Potential parallel work:

- resource loading;
- image decoding;
- CSS parsing;
- text shaping;
- isolated layout work;
- scene preparation;
- rendering;
- plugin executors.

Cross-thread results return as typed messages and are committed at deterministic synchronization points.

Immutable scene snapshots separate UI state from renderer execution.

The platform adapter decides whether actual parallelism is available, including WASM workers.

## 1.6 Failure model

Failure classes:

- API misuse;
- recoverable runtime error;
- invalid content or resource;
- unsupported capability;
- internal invariant failure.

Rules:

- panic is not normal control flow;
- failures should be localized to the smallest safe scope;
- fallback and recovery are preferred where correctness can be maintained;
- diagnostics include severity, subsystem, stable code, node/resource/window context and recovery action.

## 1.7 Component error boundaries

Every component is an error boundary by default.

This applies to lifecycle-bearing components, not every low-level retained node.

A failure in component view generation, reconciliation or local lifecycle:

- is contained within the component subtree;
- replaces the subtree with fallback UI;
- is reported to diagnostics and DevTools;
- may be retried according to component or host policy.

A component may customize or explicitly disable its boundary, except where a host-owned outer boundary is mandatory, such as plugin contributions.

Layout, renderer and resource failures use subsystem-specific recovery boundaries.

## 1.8 Security stance

Native in-process Rust code is trusted code. The kit does not claim to sandbox arbitrary native libraries.

Actual untrusted execution requires:

- a separate process;
- a WASM runtime with enforced isolation;
- capability-based services;
- explicit resource and time limits.

Restricted shader and expression environments are validated and bounded, but are not presented as equivalent to a process sandbox.
