# 9. Roadmap, Exclusions and Unresolved Details

## 9.1 Explicit non-goals

Obscur Kit 2 does not aim to provide:

- application domain architecture;
- mandatory global state store;
- application-wide undo/redo;
- process lifecycle ownership;
- a browser DOM/CSSOM clone;
- a full HTML compatibility layer;
- legacy browser layout behavior;
- arbitrary networking in the core;
- a full HTTP framework;
- a full media codec/recording stack;
- a print/page-fragmentation engine;
- a false in-process sandbox for native Rust plugins;
- transparent filesystem path access for sandboxed plugins;
- direct renderer access for ordinary plugins;
- automatic execution of business logic inside `.okui`.

## 9.2 Deferred or optional features

- network-awareness events;
- external GTK/Qt theme adapters;
- system clipboard history;
- broad media-device stack;
- media playback integrations;
- persistent package signatures and publisher trust;
- advanced remote plugin service adapters;
- full vector/PDF export depending on renderer support;
- internal ECS optimization;
- aggressive parallel layout;
- advanced shader extension APIs;
- platform-specific privileged render extensions.

## 9.3 Documents still to be specified

The architecture drafts should later be followed by narrower normative specs:

1. Rust public API conventions.
2. UI IR binary/schema format.
3. `.okui` grammar and type system.
4. CSS support matrix and computed-style rules.
5. Layout constraint and measure/arrange contracts.
6. Motion timeline/track/interpolator model.
7. Scene/Paint data model.
8. Resource manifest.
9. Diagnostics protocol.
10. Plugin manifest schema.
11. Plugin ABI function tables.
12. Plugin DTO/IDL.
13. Capability catalogue.
14. Host service contracts.
15. Headless test API.
16. Platform adapter contract.

## 9.4 Recommended implementation order

### Phase A — minimal vertical slice

- `UiRuntime`;
- retained tree;
- component lifecycle;
- event propagation;
- basic flow/flex layout;
- CSS parser and computed style;
- text shaping;
- Scene/Paint;
- Vello backend;
- one desktop platform adapter;
- headless runtime;
- diagnostics foundation.

### Phase B — production UI fundamentals

- grid;
- incremental layout;
- focus;
- IME/editing;
- scroll;
- accessibility;
- resources;
- multi-window;
- motion engine;
- DnD/data transfer;
- virtualization.

### Phase C — advanced surfaces

- Canvas;
- custom chrome;
- docking;
- infinite workspaces;
- overlay host;
- offscreen/export;
- media/foreign surfaces;
- adaptive quality.

### Phase D — plugin platform

- static contribution model;
- manifest/package loading;
- UI IR loading;
- Host API services;
- native ABI;
- WASM/process executor;
- capability policy;
- plugin SDK and DevTools integration;
- staged updates and migrations.

## 9.5 Open implementation questions

These do not block the architecture draft but require concrete design:

- exact numeric representation for geometry;
- exact CSS parser and selector engine strategy;
- whether some layout work may be parallelized safely;
- UI IR text/binary representation;
- expression bytecode or AST representation;
- chosen IDL technology or custom schema;
- default sandboxed WASM engine;
- default local IPC transport;
- exact Vello integration boundaries;
- resource cache budgets;
- accessibility adapter coverage by platform;
- capability naming and UX wording;
- exact plugin package directory structure;
- exact stable ABI memory rules;
- deterministic font strategy in headless tests.

## 9.6 Architectural guardrails

Future additions should be rejected or redesigned if they:

- leak platform types into core;
- require arbitrary mutation of retained tree;
- mix application state with view state;
- make renderer internals part of ordinary APIs;
- introduce untyped string dispatch;
- require hidden synchronous blocking;
- make CSS and motion inseparable;
- weaken host control over plugin capabilities;
- turn DevTools protocol into an arbitrary code-execution channel;
- promise sandbox guarantees that the selected execution model cannot enforce.
