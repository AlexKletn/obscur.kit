# Obscur Kit 2 — Per-Crate Implementation Plan

> Derived from the architecture drafts (`specs/obscur-kit-2-spec-drafts`) and the
> crate architecture (`specs/obscur-kit-2-crate-architecture`). Ordered by the
> implementation phases in crate-arch §5.12 and drafts §9.4. Each crate lists its
> prerequisites, ordered steps (sub-steps for large crates), and done-criteria.

## How to read this

- **Status legend:** `done` (real content) · `partial` (starter types exist) ·
  `stub` (placeholder only, full target layout present).
- **Golden rule (drafts §7.14):** every crate proves itself with backend-independent
  snapshots (UI IR / retained tree / computed style / layout / Scene / a11y) before
  any raster/GPU test. Keep the headless slice authoritative.
- **Boundary rule (crate-arch §1.11):** external types (winit/wgpu/vello/taffy/
  parley/accesskit/wasmtime) never appear in a domain public API — only inside the
  matching adapter. `cargo run -p xtask -- check` enforces the mechanical parts.
- Sub-steps are sequenced so each is independently testable.

## Crate → phase map

| Phase | Crates |
|---|---|
| 0 · enforcement (done) | workspace, lints, `xtask` |
| 1 · headless vertical slice | identity, geometry, diagnostics, task*, view, tree, style, layout*, scene, render, css*, primitives*, resources*, runtime*, headless, testkit* |
| 2 · first native window | task-smol, window-winit, render-vello, windowing, text*, resources, runtime (scheduler), obscur-kit |
| 3 · interaction & editing | interaction, data-transfer, text (editing/IME), accessibility, accessibility-accesskit, system, widgets* |
| 4 · motion & advanced layout | motion, css (transitions/keyframes), layout (grid/virtual/docking), scene (canvas/offscreen), render (offscreen), components, theme-system, render-cpu |
| 5 · language & tooling | ui-ir, okui-syntax, okui-compiler, ui-macros, devtools-protocol, devtools-server, tools/*, headless (record/replay) |
| 6 · plugins | plugin-model, plugin-api, plugin-abi, plugin-package, plugin-host, plugin-sdk, plugin-macros, plugin-executor-wasm, plugin-executor-process, tools/plugin-cli |
| — · deferred adapters | task-tokio, cache-fs, resource-fs, system-{windows,macos,web} |

`*` = begins partial in its phase and is finished in a later phase.

---

# Phase 0 — Enforcement · **done**

Workspace (resolver 3, edition 2024), centralized `[workspace.dependencies]`,
lints, `rust-toolchain.toml`, `clippy.toml`, `deny.toml`, and the architecture
`xtask` are in place. Remaining hardening, do incrementally:

- **`xtask`**: add the two checks that need `cargo metadata` rather than text scan —
  (a) full dependency-direction graph (foundation→foundation only, domain→approved
  lower domain, no cycles), (b) "internal crate must not depend on `obscur-kit`".
  Add "one file exports one main entity" via a lightweight `syn` parse.
- **CI** (crate-arch §5.10): `fmt --check`, `clippy`, `nextest`, `xtask check`,
  feature-combination matrix, `trybuild`, fuzz smoke; periodic Miri/ASan/MSRV/WASM.

---

# Phase 1 — Headless vertical slice

**Goal (crate-arch §5.13):** prove `component → desired tree → retained tree →
computed style → layout → Scene/Paint` deterministically, with **no OS and no GPU**.
This is the spine everything else hangs off; get its contracts right before breadth.

## `obscur-identity` — *partial → done*
**Depends on:** none.
**Steps:**
1. Confirm the 8 id newtypes, `Generation`, `VersionStamp`, `StableKey`,
   `InternedSymbol` (done).
2. Add an `IdAllocator<T>` pattern (generational) used by `secondary_map` consumers —
   but keep the allocator in `obscur-tree`; identity stays pure data.
3. Property tests: id round-trips, generation `next()` monotonicity/wrap.
**Done when:** `insta`/`proptest` cover id + generation invariants; zero deps.

## `obscur-geometry` — *partial → done*
**Depends on:** none.
**Steps:**
1. Finish primitives: `Vector`, `Edges`, `Normalized`, plus `unit/normalized`.
2. `transform/transform_chain` (ordered affine composition) and
   `transform/coordinate_mapper` (map a point/rect between two spaces).
3. `space/*` as **zero-sized marker types** (`LocalSpace`, `WindowSpace`, …) plus a
   `Tagged<Space, T>` wrapper so coordinate space is a compile-time tag, not a runtime
   enum (crate-arch §3.6).
4. `constraint/axis_constraint`, `aspect_ratio`; finish `snapping/pixel_snap_policy`.
5. Exact-fraction predicates: `Rect::contains`, intersection, union — **no rounding**
   (drafts §4.6).
**Done when:** proptest covers transform inverse, constrain idempotence, hit-test on
fractional geometry; `#![forbid(unsafe_code)]`.

## `obscur-diagnostics` — *partial → done*
**Depends on:** identity.
**Steps:**
1. Finish `performance/{performance_counter, budget_violation}`,
   `subscription/{diagnostic_subscription, diagnostic_buffer_policy}`,
   `port/telemetry_sink`.
2. Define the **stable diagnostic-code registry** convention (namespaced consts per
   subsystem) — codes are contract (drafts §1.6).
3. Backpressure/buffer policy for subscriptions (drop-oldest / bounded).
**Done when:** a `Vec`-backed test sink captures/filters by severity; codes documented.

## `obscur-task` — *partial → finish in Phase 2*
**Depends on:** none.
**Steps (Phase 1 slice):**
1. Add `time/duration` wrapper, `task/{task_result, stale_result}` (generation-tagged
   result envelope for discarding stale work, drafts §3.3).
2. Keep ports (`Spawner`, `LocalSpawner`, `Timer`, `Clock`, `BlockingExecutor`) as the
   only async surface; **no executor** in this crate.
**Done when:** headless deterministic clock/executor (in `obscur-headless`) satisfy
these ports; stale-result discarding unit-tested.

## `obscur-view` — *partial → done (large: sub-stepped)*
**Depends on:** identity, diagnostics.
**Sub-steps:**
1. **Contract model** — `contract/*`: `PropContract`, `EventContract`, `SlotContract`,
   `ChildContract`, `StylePartContract`, `CapabilityContract`. Typed, introspectable
   (drafts §2.13).
2. **Component trait** — `component/component`: lifecycle hooks (create/mount/update/
   unmount), typed props/events, `ComponentContext`. Default error boundary
   (`boundary/*`) contained per component (drafts §1.7).
3. **Desired view** — `view/*`: `ViewNode`, `ViewFragment`, keyed children
   (`ViewKey`), `ViewProperty`, `ViewEventBinding`. This is the declarative output; it
   must NOT reference retained nodes.
4. **Local state** — `state/*`: `LocalState`, `ReadSignal`, `SubscriptionScope` with
   auto-cleanup; values-down / actions-up only (drafts §3.1).
5. **Registry** — `component/component_registry` + `component_factory`: stable
   component id → contract + factory (drafts §2.14). Static registration only in P1.
**Done when:** a test component emits a deterministic desired tree; snapshot of
`ViewFragment`; error-boundary substitution is exercised.

## `obscur-tree` — *partial → done (large: sub-stepped)*
**Depends on:** identity, view.
**Sub-steps:**
1. **Nodes** — `node/*`: `RetainedNode`, `NodeKind`, `NodeFlags`, logical vs visual
   parent, `NodeGeneration`. Retained tree is the **sole owner of identity**
   (crate-arch §1.5).
2. **Storage** — `tree/{retained_tree, secondary_map, subtree}`: `slotmap`-backed
   generational store; secondary maps hold layout/paint/a11y projections keyed by
   `NodeId` (crate-arch §4.9). Backend handles never replace `NodeId`.
3. **Reconciliation** — `reconciliation/*`: `Reconciler`, `ReconciliationPlan`,
   `ReconciliationOp`, `ChildMatcher` (keyed identity), `IdentityMatch`. Diff desired →
   retained, preserving identity across moves.
4. **Transactions** — `transaction/*`: `TreeTransaction`, `TreeCommit`,
   `TreeChangeSet`; validated, atomic between frames (drafts §3.2).
5. **Portals/layers** — `portal/*`, `layer/*`: logical parent preserved across portal
   (style/focus scope), stacking contexts.
**Done when:** reconciliation is deterministic under proptest (insert/remove/reorder
with keys preserves ids); `TreeSnapshot` insta-snapshots are stable.

## `obscur-style` — *partial → done (large: sub-stepped)*
**Depends on:** identity, (tree for matching context via a narrow port).
**Sub-steps:**
1. **Property model** — `property/*`: `PropertyId`, `PropertyValue`,
   `InheritedProperty`, `CustomProperty`. Typed values, not strings.
2. **Selectors** — `selector/*`: `SelectorSpecificity`, `SelectorMatch`,
   `PseudoState`. (Matching against a tree adapter; the *parser* is `obscur-css`.)
3. **Cascade** — `cascade/*`: origins (system→theme→global→component→inline→runtime),
   specificity then source order, `!important` layering, runtime-override top layer
   (drafts §4.14).
4. **Custom properties + computed** — resolve custom properties before computed values;
   `computed/{computed_style, computed_value, style_snapshot, style_diff}`.
5. **Scopes** — `scope/*`: module scoping (no artificial specificity bump), theme
   scopes, exported style parts; portals keep logical scope (drafts §4.15).
6. **Invalidation** — `invalidation/*`: dependency graph + typed reasons + `StyleCache`;
   invalidate only affected consumers.
**Done when:** given a stylesheet (from `obscur-css`) + tree, `ComputedStyle` snapshots
are deterministic; cascade order + `!important` covered by table tests.

## `obscur-layout` — *partial → flow/flex in P1, rest in P4 (large: sub-stepped)*
**Depends on:** geometry, style (computed layout style), a narrow text-measure port.
**Sub-steps (Phase 1):**
1. **Contract** — `pass/*` measure→arrange; `context/*` (`LayoutContext`,
   containment, dependency), `model/*` (`LayoutStyle`, `LayoutBox`, `LayoutResult`,
   `IntrinsicSize`) (drafts §4.2).
2. **Flow/block + flex** — `algorithm/{flow_layout, flex_layout}`. Wrap **Taffy behind
   `algorithm/taffy/*`** (style mapper, tree view, result mapper); no `taffy::*` in the
   public API (crate-arch §4.4).
3. **Incremental** — `invalidation/*`: start at nearest affected context, propagate up
   only on geometry change / down only on constraint change; measure/layout caches;
   early-exit (drafts §4.3).
4. **Intrinsic port** — `port/intrinsic_measure`: layout asks text/replaced content for
   sizes; a mock provider serves the P1 slice.
**Sub-steps (deferred to Phase 4):** `grid_layout`, `positioned_layout`,
`constraint_layout`, `docking_layout`, `custom_layout`; `observer/*` (resize/visibility/
intersection); `virtualization/*`.
**Done when (P1):** flow+flex `LayoutSnapshot` deterministic; incremental relayout only
touches dirty contexts (assert via instrumentation counters).

## `obscur-scene` — *partial → done (large: sub-stepped)*
**Depends on:** geometry.
**Sub-steps:**
1. **Vector model** — `color/*`, `path/*`, `paint/*` (brushes, gradients, stroke,
   blend), `clip/*`, `layer/*` (drafts §5.1). One model shared by CSS decoration, SVG,
   Canvas, headless.
2. **Display list** — `command/*`: `PaintCommand`, `DisplayList`, `DisplayListBuilder`;
   deterministic paint order.
3. **Snapshots** — `scene/*`: immutable `SceneSnapshot` + generation + statistics; this
   is the hand-off to any renderer (crate-arch §1.6).
4. **Surfaces** — `surface/*`: descriptors for foreign/canvas/offscreen (data only;
   behaviour lands in P4).
**Done when:** building a Scene from a styled+laid-out tree yields a stable
`SceneSnapshot` insta-snapshot; color/premultiply conventions documented (drafts §5.7).

## `obscur-render` — *partial → done (port only)*
**Depends on:** scene, geometry.
**Steps:**
1. `renderer/*`: `Renderer` trait, `RendererCapabilities`, `RendererError`.
2. `target/*` (window/texture/bitmap), `frame/*` (submission, present result, device
   loss), `extension/*` (trusted custom-pass boundary) — **contracts only**, no GPU
   (crate-arch §2.3).
3. Determinism boundary spec: what must match across backends vs what may differ
   (drafts §5.2), encoded as doc + a conformance-test trait.
**Done when:** a no-op/collecting test renderer implements the port and records the
display list for assertions.

## `obscur-css` — *stub → minimal in P1, full in P4 (large: sub-stepped)*
**Depends on:** style, motion (later), (identity).
**Sub-steps (Phase 1 minimal):**
1. **Tokenize/parse** — `source/*`, `parser/{stylesheet, selector, property, value}`
   over `cssparser`; selector matching via Servo `selectors` against a tree adapter.
2. **Compile** — `compiler/{stylesheet, selector, property}`: emit `obscur-style`
   structures (not browser CSSOM). Cache per source id + version (drafts §4.12).
3. `error/*`: typed parse errors + `UnsupportedCssFeature` diagnostics for excluded
   features (drafts §4.13).
**Sub-steps (Phase 4):** `media_query_parser`, `container_query_parser`,
`compiler/{transition_compiler, keyframe_compiler}` lowering to `obscur-motion`.
**Done when (P1):** a stylesheet parses to computed style deterministically; fuzz target
on the parser; unsupported-feature diagnostics fire.

## `obscur-resources` — *partial → minimal in P1, full in P2 (large: sub-stepped)*
**Depends on:** identity, task (ports), diagnostics.
**Sub-steps (Phase 1):**
1. `handle/*` typed handles + `state/*` (loading/ready/failed, version).
2. `hub/*`: `ResourceHub`, entries, namespaces, `ResourceRequest`; dedup; stale-job
   rejection by version (drafts §5.15).
3. `source/{embedded, memory, generated}` for deterministic test resources.
**Sub-steps (Phase 2):** `port/{resource_loader, resource_decoder, resource_compiler,
cache_store}`; `cache/*` budgets/eviction; `source/document_source`; hot reload.
**Done when (P1):** embedded/generated resources resolve synchronously in headless with
deterministic versions.

## `obscur-runtime` — *partial → pipeline in P1, scheduler in P2 (large: sub-stepped)*
**Depends on:** view, tree, style, layout, scene, render, task, diagnostics.
**Sub-steps (Phase 1 — the pipeline):**
1. **UI transactions** — `transaction/*`: normalize mutations into validated `UiOps`,
   atomic commit between frames, provisional (commit/cancel) variant (drafts §3.2).
2. **Invalidation router** — `invalidation/*`: classify change kind (composition/paint/
   text/style/layout/tree/window) and route to the right coordinator (drafts §4.3).
3. **Orchestration** — `orchestration/*`: coordinators call each domain in order
   (style→layout→scene) via their contracts; **no algorithm duplication** (crate-arch
   §3.10). Text/motion coordinators wire in P2/P4.
4. **Frame pipeline** — `frame/frame_pipeline`: the synchronous, short, deterministic
   frame; `FrameReason`, `FrameContext`, `FrameResult` (drafts §3.4).
5. **Recovery** — `recovery/*`: component vs subsystem boundaries (drafts §1.7).
**Sub-steps (Phase 2):** `scheduler/*` (event-driven, priorities, coalescing,
backpressure, starvation guard, per-window budgets, vsync hook); `lifecycle/*`
(shutdown request/concern, runtime≠process); `quality/*` adaptive.
**Done when (P1):** driving the pipeline headless from a component produces a stable
Scene snapshot; commit boundaries + invalidation classification unit-tested.

## `obscur-headless` — *partial → done (large: sub-stepped)*
**Depends on:** runtime, task, identity.
**Sub-steps:**
1. **Deterministic core** — `deterministic/*`: clock (done), `DeterministicExecutor`,
   `CompletionQueue` (controlled async completion) (drafts §7.4).
2. **Host + virtual window** — `host/*`: drive the runtime without OS/GPU; configurable
   viewport/DPI/theme/locale.
3. **Synthetic input** — `input/*`: by coordinates and by semantic selector; typed
   dispatch results (target/path/default action/focus/state/layout changes).
4. **Pipeline snapshots** — `snapshot/*`: dump retained tree / computed style / layout /
   Scene / a11y with a normalizer for stable diffs.
5. `replay/*` — deferred to Phase 5.
**Done when:** the whole P1 slice is asserted through this host; snapshots are the
project's regression backbone.

## `obscur-testkit` — *stub → grows every phase*
**Depends on:** headless, (later) plugin-host.
**Steps:** `selector/semantic_selector`, `fixture/test_fixture`, `mock/mock_provider`,
`assertion/tree_assertion`; `host/plugin_test_host` in Phase 6. Production logic never
lives here (crate-arch §2.4).
**Done when:** downstream crates write tests via testkit, not raw runtime plumbing.

## `obscur-primitives` — *stub → partial in P1, more in P4*
**Depends on:** view, tree, style, layout, scene.
**Steps (P1):** `container`, `text` (via text-measure port + mock), `image` (static),
`portal`, `layer`. **Deferred (P4):** `scroll`, `canvas`, `foreign_surface`.
**Done when:** a small primitive tree renders to a Scene snapshot headless.

## `obscur-kit` (facade) — *stub → partial in P1*
**Depends on:** the public/default crates.
**Steps:** curate a minimal `prelude`; `bootstrap/kit_bootstrap` builds a headless
runtime in P1 (native default features arrive in P2). Never a dependency of internal
crates (crate-arch §2.10).

---

# Phase 2 — First native window

**Goal:** first interactive on-screen frame. Introduce adapters strictly behind ports.

## `obscur-task-smol` — *stub → done*
**Depends on:** task (ports).
**Steps:** implement `Spawner`/`LocalSpawner`/`Timer`/`Clock` over `async-task` +
`async-channel`; bounded queues for backpressure (crate-arch §4.8). Default executor.
**Done when:** runtime runs on a real event loop; loom tests on the queue.

## `obscur-windowing` — *stub → done (large: sub-stepped)*
**Depends on:** geometry, capability, identity.
**Sub-steps:**
1. `window/*` model + `request/*` (create/update/activate/close, typed results).
2. `display/*` (monitors, scale, refresh, HDR capability, hotplug; honest Wayland
   global-coordinate absence, drafts §6.6).
3. `chrome/*` client-side decorations & semantic regions; `presentation/*` fullscreen/
   presentation modes.
4. `port/window_backend`: the single seam adapters implement. Capability results are
   `native/emulated/conditional/unsupported` (drafts §6.2).
**Done when:** headless virtual-window maps to the same model; capabilities typed.

## `obscur-window-winit` — *stub → done*
**Depends on:** windowing (port), task.
**Steps:** implement `WindowBackend`; `conversion/*` translate winit keyboard/pointer/
window/display events into domain events; `wake/event_loop_waker`. **No `winit::*` in
any domain API** (crate-arch §2.7).
**Done when:** a real window opens, emits domain input; xtask confirms no winit leakage.

## `obscur-render-vello` — *stub → done (large: sub-stepped)*
**Depends on:** render (port), scene.
**Sub-steps:**
1. `renderer/*` implement `Renderer` over Vello/wgpu; device/surface/frame lifecycle.
2. `conversion/*` Scene/Paint → Vello (path/brush/color/clip/scene) — **local to this
   crate** (crate-arch §4.3). `vello::Scene` is not the public Scene.
3. `cache/*` glyph/texture/path caches with budgets + DevTools metrics hooks.
4. `extension/vello_custom_pass` trusted-only.
**Done when:** the P1 Scene snapshot renders on GPU; golden raster tests with tolerance
(drafts §5.2); capabilities/fallbacks reported as diagnostics.

## `obscur-text` — *stub → shaping in P2, editing/IME in P3 (large: sub-stepped)*
**Depends on:** geometry, resources (fonts), diagnostics.
**Sub-steps (Phase 2 — shaping/layout):**
1. `font/*` incl. `FontProvider` (system fonts native, bundled fallback for WASM).
2. `shaping/*` behind `TextShaper` (Parley/HarfRust/Skrifa internal); bidi, fallback.
3. `layout/*` line breaking, alignment, overflow, hit-test; satisfies layout's
   intrinsic-measure port.
4. `span/*` rich spans + CSS typography integration.
**Sub-steps (Phase 3 — editing):** `selection/*` (caret/affinity/geometry),
`editing/*` (buffer, edit transactions, deletion/navigation units, local undo),
`ime/*` (composition lifecycle, preedit, hints, drafts §5.12), `service/*` optional
language services.
**Done when (P2):** shaped `TextLayout` snapshots stable across scripts/bidi; Parley
types never escape.

## `obscur-resources` (finish) · `obscur-runtime` (scheduler)
Complete the Phase-2 sub-steps listed under each crate above (loader/cache ports; the
event-driven scheduler, vsync, per-window budgets). Wire `theme` package hot-swap
(transactional, last-good fallback, drafts §5.21).

## `obscur-kit` (native defaults)
Enable default features `window-winit`, `render-vello`, `task-smol`; `bootstrap` opens
a real window (crate-arch §5.3 feature policy).

---

# Phase 3 — Interaction & editing

## `obscur-interaction` — *stub → done (large: sub-stepped)*
**Depends on:** tree, geometry, identity, accessibility (action routing).
**Sub-steps:**
1. **Events** — `event/*`: capture→target→bubble→default, `stop_propagation`/
   `stop_immediate`/`prevent_default`, trusted origin; typed, no string dispatch
   (drafts §2.3).
2. **Pointer** — `pointer/*`: unified mouse/touch/pen; capture per pointer-id +
   registry; hover model (`hover/*`) with no synthetic touch-hover (drafts §2.4).
3. **Hit testing** — `spatial/*`: incremental spatial index, hit chain mapping visual→
   logical, spatial navigation, gamepad (`gamepad/*`). Shared with DevTools inspect
   (drafts §4.7).
4. **Focus** — `focus/*`: focus tree, scopes, tab order, programmatic focus,
   focus-visible, restore, modal traps (drafts §2.5). Focus is not a CSS flag.
5. **Commands** — `command/*`: physical→logical→shortcut→semantic command; routing
   focused→scope→ancestors→window→app (drafts §2.6). IME separate from keydown.
6. **Gestures & selection** — `gesture/*` arena; `selection/*` reusable non-text
   selection (drafts §2.7).
**Done when:** headless synthetic input yields typed dispatch results with full
propagation paths; focus traps + command routing table-tested.

## `obscur-data-transfer` — *stub → done*
**Depends on:** interaction, identity.
**Steps:** `offer/*` (`DataOffer`, MIME reps, lazy providers), `clipboard/*` provider
contract, `drag/*` (session/source/target/effect/preview/autoscroll). DnD may open a
**provisional layout transaction** for placeholders (drafts §6.21). System clipboard/
native DnD are adapter concerns.
**Done when:** internal drag round-trips headless with typed payload + MIME.

## `obscur-accessibility` — *stub → done*
**Depends on:** tree, interaction.
**Steps:** `semantic/*` (role/state/value/relation/action), `tree/*` (semantic tree +
snapshot + update), `event/*` a11y actions enter the command model, `port/
accessibility_bridge` (drafts §7.1).
**Done when:** semantic snapshot mirrors retained tree; a11y actions dispatch like input.

## `obscur-accessibility-accesskit` — *stub → done*
**Depends on:** accessibility (port).
**Steps:** implement bridge; `conversion/*` role/node/action/tree-update to AccessKit.
No `accesskit::*` upward (crate-arch §4.7).
**Done when:** platform AT sees the tree on ≥1 desktop target.

## `obscur-system` — *stub → done (large: sub-stepped)*
**Depends on:** capability, task, identity.
**Sub-steps:** independent provider contracts (not one monolith, crate-arch §2.3):
1. `document/*` `DocumentHandle`/`DirectoryHandle`/`DocumentService` (path-optional,
   portal/cloud/virtual, drafts §6.10).
2. `dialog/*` provider chain with origin/loop detection (drafts §6.9).
3. `shell/*`, `notification/*`, `capture/*` + `privacy/*`, `power/*`.
4. `network/*` `NetworkService` contract (not an HTTP framework, drafts §8.53).
**Done when:** each provider has a headless mock; capability results typed. Native
providers land per-target (linux first; windows/macos/web deferred).

## `obscur-widgets` — *stub → basic in P3, full in P4*
**Depends on:** primitives, interaction, text, accessibility.
**Steps (P3):** `button`, `input` (on text editing engine), `checkbox`, `select`,
`menu`, `dialog`, `tabs` — headless behaviour first, theming optional (crate-arch §2.6).
**Deferred (P4):** `list`, `tree`, `table`, `docking` on virtualization/collections.

---

# Phase 4 — Motion & advanced layout

## `obscur-motion` — *stub → done (large: sub-stepped)*
**Depends on:** geometry, style (base values), task (clock).
**Sub-steps:**
1. **Values** — `value/*`: `Interpolatable`, `MotionValue`, typed interpolation.
2. **Timelines/tracks** — `timeline/*`, `track/*` (keyframe/transition/spring).
3. **Drivers** — `driver/*`: time/progress/signal/scroll; interruption/reverse/chaining.
4. **Transitions** — `transition/*`: shared-element + layout transitions using before/
   after geometry; `phase/*` pre- vs post-layout sampling (drafts §4.17).
5. **Scheduler** — `scheduler/*`: sample from time/value, never slow to missed frames.
**Done when:** motion sampled at deterministic clock ticks yields stable presentation
values; interruption/reversal covered. Wire the runtime motion coordinator (pre-layout
sample → layout → post-layout resolve → presentation values).

## `obscur-css` (finish) — transitions/keyframes
Add media/container query parsers; lower `transition`/`@keyframes`/`animation-*` into
`obscur-motion` timelines (drafts §4.17). Golden: CSS animation → native timeline.

## `obscur-layout` (finish) — advanced models
Implement grid, positioned, constraint, docking, custom layout; the `observer/*`
family (post-commit, coalesced, non-reentrant, drafts §4.4); `virtualization/*`
(materialization, overscan, estimated geometry, scroll anchoring).

## `obscur-scene` / `obscur-render` (finish) — canvas & offscreen
Scene `surface/*` behaviour: Canvas participates in layout/clip/hit-test/composition
(drafts §5.3); offscreen/texture/bitmap targets + optional vector export (drafts §5.6);
`clip/mask` offscreen passes only when required. `render` gains offscreen readback.

## `obscur-components` (finish) · `obscur-theme-system` · `obscur-render-cpu`
Complete collection widgets on virtualization. `theme-system`: tokens + light/dark/
high-contrast presets + assets (a resource package, not a domain, crate-arch §2.6).
`render-cpu`: software `Renderer` for headless raster + CI without a GPU.

---

# Phase 5 — Language & tooling

## `obscur-ui-ir` — *stub → done (large: sub-stepped)*
**Depends on:** identity, capability, diagnostics.
**Sub-steps:**
1. `version/*` independent versioning + ranges (drafts §2.15).
2. `document/*`: typed IR nodes/props/events/expressions on stable component/resource/
   style ids (not Rust type names).
3. `source/*` source maps; `validation/*` validate + capability requirements + precise
   diagnostics for newer unsupported IR.
4. `migration/*` where practical; `codec/*` text+binary (codec kept behind the port,
   crate-arch §5.14).
**Done when:** IR round-trips through codec; validation rejects incompatible versions
with actionable diagnostics; fuzz the decoder.

## `obscur-ui-macros` — *stub → done*
**Depends on:** ui-ir, view.
**Steps:** proc-macro TSX-like DSL (`syn`/`quote`); component-contract derives; compile-
time UI IR embedding + source maps. **No runtime logic in the proc-macro crate**
(crate-arch §2.5). `trybuild` pass/fail suites.
**Done when:** the DSL is the primary authoring path; unknown attr/event/string method =
compile error (drafts §2.13).

## `obscur-okui-syntax` → `obscur-okui-compiler` — *stub → done (large)*
**Depends on:** ui-ir; syntax on `logos`+`rowan`.
**Sub-steps (syntax):** lexer, lossless `rowan` tree, `parser/*`, `ast/*` (element/
attribute/expression/conditional/loop/template).
**Sub-steps (compiler):** `symbol/*` + `scope`; `resolution/*`; `typing/*` type checker;
`validation/*` contract + **expression allow/deny list** (drafts §2.10) + capability
validation; `lowering/*` to UI IR with source maps. No embedded scripting (drafts §2.9).
**Done when:** `.okui` compiles to the same UI IR as the DSL; insta-snapshot the IR;
fuzz the parser.

## `obscur-devtools-protocol` / `obscur-devtools-server` — *stub → done*
**Depends on:** protocol on identity/diagnostics/scene contracts; server on runtime.
**Steps:** protocol handshake/version-negotiation/snapshots/subscriptions/commands/
permissions/errors (crate-arch §2.8). Server: local transport, **token auth**, separated
read/subscribe/control/inject permissions, backpressure, audit; disabled by default,
excludable from production (drafts §7.6–§7.7). Never arbitrary code execution.
**Done when:** external CLI inspects a live runtime read-only; control ops gated + audited.

## `tools/obscur-devtools` · `tools/obscur-okui-lsp` — *bin stubs → MVP*
DevTools CLI over the protocol; `.okui` LSP over syntax+compiler (rowan-based edits,
diagnostics). `anyhow` allowed here (crate-arch §1.7).

## `obscur-headless` (finish) — record/replay
`replay/*`: capture normalized events/scheduler decisions/clock ticks/completions;
replay against deterministic clock/executor; redact secrets by default (drafts §7.13).

---

# Phase 6 — Plugins

> Start only after UI IR, component registry, runtime transactions, resource namespaces
> and diagnostics scopes are stable (crate-arch §2.12). Build model→api→abi/package→host.

## `obscur-plugin-model` — *stub → done (large: sub-stepped)*
**Depends on:** identity, capability, ui-ir. Pure versioned DTO (crate-arch §2.9).
**Sub-steps:** `identity/*` (principal = plugin-id + install identity); `manifest/*`
(+ host compatibility, dependencies); `capability/*` (request/grant/delegated/
`UserActivationToken`); `contribution/*` + `extension_point`; `lifecycle/*`; `service/*`;
`message/*` (query/command/event envelopes, delivery semantics); `state/*`; `settings/*`.
Stable field ids, tagged unions, unknown-optional-ignored / unknown-required-error
(drafts §8.26). `schemars` JSON Schema for manifests.
**Done when:** manifests (de)serialize + validate; schema generated; compatibility rules
table-tested.

## `obscur-plugin-api` — *stub → done*
**Depends on:** plugin-model.
**Steps:** modular Host API services (`Ui`/`Action`/`Navigation`/`Resource`/`Storage`/
`SecureStorage`/`Document`/`Network`/`Clipboard`/`Notification`/`Diagnostics`), Plugin
API, `handle/*` (opaque, scoped, lease, revocable), `PluginExecutor` contract,
`service/*` registry, status/errors (drafts §8.27–§8.30). Each service versions
independently, may be absent → `unsupported`.
**Done when:** a mock host + mock plugin exchange typed messages via the API only.

## `obscur-plugin-abi` — *stub → done (unsafe crate)*
**Depends on:** plugin-api, plugin-model.
**Steps:** `repr(C)` function tables, `value/*` (abi string/buffer/slice/handle/status),
`ownership/*` rules, `boundary/*` **panic boundary + call guard** (no panic across ABI),
`conversion/*` glue (drafts §8.4). `SAFETY.md`, `unsafe_op_in_unsafe_fn`, Miri/ASan.
**Done when:** version/capability negotiation across the C ABI; no panic escapes; fuzz
the buffer/string boundary.

## `obscur-plugin-package` — *stub → done (security-critical, large)*
**Depends on:** plugin-model.
**Sub-steps:** `archive/*` zip (`zip2`) + tar (`tar >= 0.4.45`) + `flate2`/`zstd`
streaming readers; `path/*` normalize, reject `..`/absolute/root-escape/symlink;
`validation/*` limits (file count/size/ratio, archive-bomb), `blake3` hashes,
entrypoint-from-manifest-only; `extraction/*` isolated dir, no partial activation
(drafts §8.6). Fuzz every entry path.
**Done when:** malicious archives are rejected pre-execution; fuzz corpus green.

## `obscur-plugin-host` — *stub → done (largest: sub-stepped)*
**Depends on:** plugin-api, plugin-package, runtime, ui-ir.
**Sub-steps:**
1. **Discovery/catalog** — `manager/*`, `discovery/*`; index manifests without running
   code (drafts §8.35 lazy activation).
2. **Dependencies** — `dependency/*`: graph, resolver, conflict diagnostics, cycles
   rejected (drafts §8.31).
3. **Policy** — `policy/*`: host overrides manifest; capability/activation/budget policy
   (drafts §8.10).
4. **Activation** — `activation/*`: transactional, partial activation per contribution,
   rollback on failure (drafts §8.11, §8.34).
5. **Contributions** — `contribution/*`: registry + instances + typed context +
   **mandatory host-owned error boundary** + host chrome + extension-point registry
   (drafts §8.14–§8.18, §8.39).
6. **Capabilities** — `capability/*`: issue/revoke/delegate (derived ≤ source, audited,
   drafts §8.28–§8.29, §8.58).
7. **Lifecycle** — `lifecycle/*`: teardown order, suspension, crash recovery/backoff,
   native-unload safety (drafts §8.12–§8.13, §8.61).
8. **Update** — `update/*`: state migration, staged handover, rollback (drafts §8.44–
   §8.45); `storage/*` namespaced/quota/orphans; `performance/*` watchdog + budgets.
**Done when:** headless plugin test host loads a package, activates contributions into a
runtime subtree, enforces capabilities, survives a plugin crash, and does a staged
update with no empty panels.

## `obscur-plugin-executor-wasm` / `-executor-process` — *stub → done*
**Depends on:** plugin-api (executor port).
**Steps (wasm):** Wasmtime behind `PluginExecutor`; fuel/epoch/memory limits; no WASI
fs/net without host capability; no Wasmtime types in the API (drafts §8.23).
**Steps (process):** framed messages + bounded queues over UDS/named pipes (no shared
memory first). Both: message-based, restartable, host-owned fallback.
**Done when:** a sandboxed plugin runs UI-IR contributions; resource limits enforced;
kill/restart restores last committed state.

## `obscur-plugin-sdk` / `obscur-plugin-macros` / `tools/obscur-plugin-cli`
SDK: ergonomic wrappers hiding `unsafe`/glue; `contribution_builder`. Macros: manifest/
service/contribution/ABI-glue generation. CLI: package/validate/mock-host/inspect UI IR
& contracts (drafts §8.63).
**Done when:** a template plugin builds, packages, and loads end-to-end via the CLI.

---

# Cross-cutting workstreams (continuous)

- **Benchmarks (crate-arch §5.11):** reconciliation, selector/cascade, style/layout
  invalidation, flow/flex/grid, shaping, scene building, display-list cache, renderer
  submission, resource cache, plugin codecs/executors — each measuring the domain op
  apart from adapter overhead.
- **Fuzzing:** CSS, `.okui`, UI IR, plugin manifest, zip/tar entry validation, SVG/font/
  image boundaries.
- **Determinism debt:** every new subsystem ships headless snapshots first; raster/GPU
  goldens only with tolerances.
- **Reversible decisions (crate-arch §5.14):** keep Taffy depth, Vello version, Parley
  internals, default executor, UI-IR/plugin codecs, Wasmtime choice, IPC transport,
  cache format, diagnostics transport all behind ports.
- **Stabilize early (crate-arch §5.15):** typed ids, coordinate spaces, component
  contract, event propagation, transaction boundaries, cascade semantics, measure/
  arrange contract, Scene/Paint model, capability results, diagnostic codes, UI-IR
  version rules, contribution/extension-point semantics.
