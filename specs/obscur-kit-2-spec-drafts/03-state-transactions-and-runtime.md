# 3. State, Transactions and Runtime

## 3.1 View state

State enters UI through:

- typed props;
- read-only signal/observable sources;
- local component view state.

The toolkit does not provide a mandatory global store.

Scoped subscriptions are automatically cleaned up.

Dependency tracking invalidates only consumers of changed values.

No magic two-way binding is provided:

- values flow down;
- typed actions/commands flow up.

## 3.2 UI transactions

Mutations are normalized into validated `UiOps` and committed atomically between frames.

Useful concepts borrowed from intent-oriented architecture:

- explicit command/intent;
- validation and policy;
- normalized operations;
- reducer;
- transactional commit;
- effective projections.

The toolkit does not recreate a full application Intent Machine, sync engine or global overlay model.

Rollback is scoped:

- normal UI commits are validated before application;
- provisional systems such as DnD may use commit/cancel semantics;
- heavyweight universal rollback is not required.

## 3.3 Async abstraction

The core is not tied to Tokio.

Runtime adapters provide:

- `Spawner`;
- `Timer`;
- `Clock`;
- cancellation;
- optional blocking-work facility.

Rules:

- tasks are scoped to component, window or runtime;
- scoped tasks are cancelled automatically;
- results return as typed messages;
- stale work is discarded using generation/version IDs;
- `spawn_blocking` is explicit and capability-dependent;
- headless mode can use deterministic executors and clocks.

## 3.4 Frame pipeline

Conceptual pipeline:

1. receive platform events and async completions;
2. choose frame reasons;
3. dispatch input/events;
4. commit state/tree transaction;
5. classify invalidation;
6. resolve styles;
7. sample pre-layout motion;
8. perform incremental layout;
9. resolve post-layout motion;
10. build paint/display lists;
11. build compositor/render graph;
12. submit/present;
13. publish post-frame notifications.

The synchronous frame phase must stay short and deterministic.

## 3.5 Frame scheduler

The scheduler is event-driven.

Frame reasons include:

- input;
- UI commit;
- animation;
- resource completion;
- window change;
- external render request;
- diagnostics capture.

Rules:

- no busy loop;
- updates are coalesced;
- frame callbacks/vsync are used where available;
- scheduling is per window where useful;
- active animations continue requesting frames;
- hidden/minimized windows may suspend by policy;
- stale intermediate frames may be skipped;
- animation progress is sampled from time/value, never slowed to match missed frames.

## 3.6 Scheduler priorities

Priority order:

1. input, focus, pointer and IME;
2. UI transaction commit;
3. current-frame animation, layout and paint;
4. resource completion;
5. background shaping, decoding and cache work;
6. telemetry and development diagnostics.

The scheduler supports:

- coalescing;
- stale-task cancellation;
- deadlines;
- starvation prevention for background work;
- per-window and per-subsystem budgeting.

## 3.7 Visibility and lifecycle

Runtime lifecycle does not imply process lifecycle.

The runtime emits events such as:

- last window closed;
- shutdown requested;
- critical stop;
- resource release complete.

The host decides whether to exit.

A runtime may stop and release resources while the process continues.

## 3.8 State persistence helpers

Optional versioned helpers may serialize and restore:

- window geometry and mode;
- scroll positions;
- selected tabs;
- docking workspace;
- expanded tree nodes;
- safe focus restoration.

The application chooses:

- which state is persisted;
- where it is stored;
- when it is restored.

The kit does not own application persistence.
