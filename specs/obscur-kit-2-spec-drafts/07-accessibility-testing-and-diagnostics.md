# 7. Accessibility, Testing and Diagnostics

## 7.1 Accessibility tree

Accessibility uses a semantic tree linked to the retained UI tree.

Semantic data:

- role;
- name;
- description;
- state;
- value;
- actions;
- relations;
- focus;
- selection;
- decorative hiding.

Platform adapters may target:

- UI Automation;
- AX;
- AT-SPI;
- web accessibility APIs.

Accessibility actions enter the same command/event model as ordinary input.

## 7.2 Localization

Localization abstraction includes:

- key;
- parameters;
- current locale;
- fallback;
- replaceable provider;
- RTL integration;
- text shaping integration.

Default implementation should be non-Anglocentric and support, at minimum:

- Russian plural categories;
- numeral agreement;
- gender;
- case variants;
- locale-aware dates;
- numbers;
- currencies.

The toolkit is not a full application translation-management platform.

## 7.3 RTL

RTL behavior relies on:

- logical CSS properties;
- bidi text;
- semantic mirroring;
- layout-aware direction;
- direction propagated through theme/environment.

## 7.4 Headless runtime

Official headless mode operates without window or GPU.

It can output:

- retained tree;
- computed styles;
- layout;
- scene;
- paint;
- accessibility tree;
- render graph;
- rendered image.

Testing facilities:

- deterministic clock;
- deterministic executor;
- synthetic input by coordinates;
- synthetic input by semantic selector;
- typed dispatch results;
- reproducible resources;
- controlled async completion.

Dispatch results may include:

- target;
- propagation path;
- default action;
- focus changes;
- state changes;
- layout changes.

## 7.5 Stable selectors

Testing and DevTools use semantic selectors independent of CSS.

Available identifiers:

- `test-id`;
- `automation-id`;
- semantic role/name;
- stable component/node ID.

These identifiers:

- do not affect layout;
- do not affect CSS specificity;
- do not affect visual output;
- may be compiled out of production builds.

## 7.6 External DevTools

DevTools is an external CLI/GUI, not a mandatory built-in application UI.

It connects through a stable diagnostics protocol over IPC/RPC/local socket/platform equivalent.

Inspectable live state:

- tree;
- styles;
- layout;
- event flow;
- focus;
- animations;
- resources;
- accessibility;
- scene;
- rendering;
- caches;
- plugin state.

Inspect mode uses the same hit-testing engine as runtime and can highlight bounds.

## 7.7 Diagnostics endpoint security

The diagnostics endpoint:

- is disabled by default;
- is local-only unless explicitly configured;
- authenticates with token/capability;
- separates read, subscribe, control and event-injection permissions;
- supports protocol version negotiation;
- enforces message/subscription limits;
- audits dangerous operations;
- can exclude control features entirely from production.

The endpoint never provides arbitrary Rust execution or unrestricted memory access.

## 7.8 Diagnostics channel

Recoverable errors and warnings can be:

- subscribed to;
- filtered;
- buffered;
- backpressured;
- disabled.

Diagnostics are typed and structured rather than plain logs.

## 7.9 Performance telemetry

Spans and counters may cover:

- dispatch;
- reconciliation;
- CSS;
- text;
- layout;
- animation;
- hit testing;
- paint;
- render;
- CPU/GPU;
- resources;
- caches.

Attribution:

- frame;
- window;
- subsystem;
- node;
- layout context;
- resource;
- invalidation cause;
- plugin contribution where applicable.

Budgets produce diagnostics, not crashes.

Deep tracing is opt-in. Lightweight counters may remain available in production only when explicitly enabled.

## 7.10 Adaptive quality

Optional adaptive-quality policy may reduce:

- noncritical blur;
- shadows;
- filters;
- raster cache resolution;
- background Canvas quality;
- virtualization overscan.

It must not degrade:

- input correctness;
- text correctness;
- focus;
- selection;
- accessibility;
- final animation state.

Changes are visible in telemetry.

## 7.11 Power awareness

Optional `PowerProfile` may influence:

- background work;
- visual effects;
- telemetry;
- polling.

It is off by default and must not compromise critical responsiveness.

## 7.12 Diagnostic snapshots

Development/diagnostic mode may serialize snapshots of:

- retained tree;
- computed styles;
- layout geometry;
- focus;
- selection;
- scroll;
- animation;
- accessibility;
- scene/paint state.

Uses:

- DevTools;
- reproducible bug reports;
- headless tests;
- frame comparison.

Full runtime restoration is not required.

Snapshot/protocol code may be entirely excluded from production builds.

## 7.13 Record and replay

Optional development/test record-replay may capture:

- normalized platform events;
- scheduler decisions;
- clock ticks;
- async completions;
- resource versions;
- window/environment changes.

Replay runs against deterministic clocks/executors.

Sensitive input such as passwords, secrets and protected clipboard data is excluded or redacted by default.

## 7.14 Golden testing

Golden tests should prefer backend-independent snapshots:

- UI IR;
- retained tree;
- computed style;
- layout;
- Scene/Paint;
- accessibility.

Raster golden tests are allowed with explicit tolerances for anti-aliasing and font/backend variation.
