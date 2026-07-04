# obscur.kit_v2 — Spec Conformance Audit

- **Date:** 2026-07-04
- **Commit:** `355cc29`
- **Workspace:** ~2674 tests passing, `cargo run -p xtask -- check` clean.
- **Method:** read-only audit by 7 parallel agents, one per spec area, each mapping
  `specs/obscur-kit-2-spec-drafts/*` + `specs/obscur-kit-2-crate-architecture/*`
  requirements to the implementation with `file:line` evidence. No code changed.
- **Status legend:** ✅ conformant & wired · 🟡 partial · 🔵 modeled/deferred by design (documented) · ⬜ port-only / stub · ❌ missing or violates spec.

---

## Executive verdict

Architecturally the toolkit is **strongly spec-faithful** (hexagonal boundaries hold, 55/55 catalog crates exist, layering is clean, ports are object-safe) and the **domain primitives are complete and well-tested**. The dominant finding is a single cross-cutting theme:

> **"Built but not wired."** Many subsystems are individually complete and unit-tested but are **not connected into the live frame pipeline / runtime**. The examples work only because `HeadlessHost` hand-wires a minimal subset.

Secondary themes: the **value model (`PropertyId` = 7 properties) is far narrower than the engines operating on it**, the **`UiRuntime` object owns nothing** (integration lives in the headless test host), and several spec-required subsystems are **honest stubs** (devtools protocol/server, diagnostics telemetry, permission decisions, resource decoders, real text shaper/fonts, plugins).

There is **one concrete spec violation** (small/cheap to fix): §2.13 "unknown attributes/events are compile errors" is unenforced.

---

## Cross-cutting meta-findings

### M1 — "Built but not wired" (spans nearly every area)

| Subsystem | Built | Not connected to |
|---|---|---|
| UI/tree **transactions** | ✅ primitives + tests | `FramePipeline` mutates the tree directly via `reconcile_root` (`obscur-runtime/src/frame/frame_pipeline.rs:96`); §3.2 "atomic commit between frames" is proven only in unit tests |
| **InvalidationRouter** + **FrameScheduler** | ✅ complete + tested | no pipeline/runtime code calls them (`invalidation/invalidation_router.rs:109`, `scheduler/frame_scheduler.rs`) → incremental invalidation never happens; all 4 phases run every frame |
| **obscur-text** (shaping/layout/selection/IME) | ✅ whole domain | render path bypasses it: `PaintCommand::DrawText` carries a raw string (`obscur-scene/.../paint_command.rs:43-52`), runtime has no `obscur-text` dep, shaping is naive skrifa in the vello adapter (`render-vello/.../scene_converter.rs:156-204`); CPU backend renders no text → **text determinism broken** |
| **Input** (pointer/keyboard converters) | ✅ complete + tested | **zero non-test callers**; winit `window_event` only runs the lifecycle converter (`window-winit/.../winit_event_loop.rs:193`) → interaction pipeline receives no real input |
| **pseudo-state / `[data-*]`** | matcher supports them | `TreeElement` always returns `PseudoState::empty()` and only `id`/`type` attributes (`obscur-runtime/.../tree_element.rs:107-121`) → `:hover`/`:focus`/`[data-*]` never match at runtime |
| **Motion orchestration** (interruption/chaining/sync/phases; style→motion→paint) | math core ✅ | not enacted (`motion/scheduler/motion_scheduler.rs:58-101`; interruption is a policy enum no code applies) |
| **Style scopes / CSS modules / generated parts** | data model | `StyleMatcher` ignores scope; `::before`/`::after`/scrollbar/selection are never produced (`obscur-tree/.../node_kind.rs:28` has `Synthetic` but no producer) |

### M2 — Value model narrower than the engines

- **`PropertyId` = 7 fixed variants** (Color, BackgroundColor, Opacity, Width, Height, Display, Visibility — `obscur-style/src/property/property_id.rs:20-35`). The CSS compiler allowlists exactly these; every other property → `Unsupported(Property)`. Most of §4.13 (margin/padding/font/transform/overflow/shadow/filter/cursor/logical-props/flex-item) is unreachable as computed style.
- **`display` hardcoded to Block** in `layout_style_from` (`obscur-runtime/.../layout_coordinator.rs:143-150`) → flex/grid/positioned never engage from CSS; even `LayoutDisplay` has only Block/Flex/None.
- `calc/min/max/clamp` parsed but never evaluated; `Length` is px/%/auto only (no em/rem/vw/vh). `var()` **is** resolved with a cycle guard (bright spot — `computed/style_resolver.rs:206-228`).
- `@media`/`@container` conditions parsed but **gating never applied** — rules lifted unconditionally into the matchable set (`css/.../stylesheet_compiler.rs:330-343`).

### M3 — `UiRuntime` owns nothing

`runtime/ui_runtime.rs:11` is `struct UiRuntime { id: RuntimeId }`; the catalog mandates it own the frame pipeline, scheduler, UI transactions, invalidation, lifecycle. The only real integrator is `HeadlessHost`. §3.7 lifecycle (last-window/shutdown/critical-stop) and §3.3 stale-result discard are placeholder structs.

---

## Strengths (solidly conformant)

- **Architecture & boundaries:** layering respected (foundation→foundation, no domain→adapter, nobody depends on the facade), external platform crates confined to adapters, `taffy` confined to `obscur-layout/algorithm/taffy/`, cargo policies (resolver 3 / edition 2024 / workspace lints / `#![forbid(unsafe_code)]`), facade has no domain logic.
- **CSS cascade (§4.14):** origin/specificity/source-order/`!important`/runtime-override, custom-property participation, `var()` resolution — real and tested.
- **Selector matcher (§4.13):** full surface — combinators with backtracking, `:nth-*` An+B, all 6 attribute operators, pseudo-classes.
- **Layout algorithms:** 7 real (flow/flex/grid/positioned/constraint/docking/custom) + typed incremental invalidation model.
- **Motion core:** interpolation, timeline integration (skipped-frame safe), keyframe/spring/transition tracks, FLIP math, 4 drivers.
- **Interaction domain:** event dispatch (capture/target/bubble), focus tree + tab order + restore, gesture arena, spatial hit-test, selection model, command router, routing, data-transfer/DnD.
- **Renderer port (§5.2):** object-safe, honest capability negotiation (`SupportLevel`), typed errors, no silent no-ops, host-provided device.
- **Resources:** `ResourceHub` real state machine (dedup/stale/hot-reload) + persistent fs `CacheStore` (versioned, atomic rename, cross-run tested).
- **Text editing/IME (§5.11-5.12), a11y semantic tree + AccessKit bridge, headless determinism** (deterministic clock/executor, golden snapshots).
- **Plugins/non-goals:** clean deferral; no non-goal violated (okui never executes logic; CSS never fetches `@import`/`url()`).

---

## Area 1 — Architecture & boundaries (spec 01 + arch docs)

Mostly conformant. Gaps:

- 🟡 **xtask weaker than §5.9:** `dependency_direction` is a string-scan for 8 external crate names in 3 pure dirs, not a cargo-metadata graph. Would not catch a hypothetical foundation→domain or domain→internal-adapter edge; no cycle check, no "must not depend on facade" check, no feature-matrix check. Graph is correct today by hand.
- 🟡 `[workspace.dependencies]` not fully centralized (omits testkit / plugin-* / devtools / system-macos+windows — all empty stubs).
- 🟡 `repository` is a placeholder URL; `default-members` points at obscur-runtime not the facade.
- 🔵 `obscur-ui-macros` not `proc-macro=true` (logic crate; rustc's crate-root requirement collides with the module_layout rule). Defensible.
- 🔵 Recommended externals not all used: css hand-rolled (no cssparser/selectors), no serde in ui-ir, no slotmap in tree, tiny-skia instead of vello_cpu. §4 is explicitly recommendations; hand-rolling reinforces "no external types in public API".

---

## Area 2 — Components, tree, DSL, contracts, UI IR (spec 02)

- ❌ **§2.13 "unknown attributes/events are compile errors" — unenforced.** The only contract check (`okui-compiler/.../contract_validator.rs:115-156`) verifies required-props-present + allowed-children; nothing rejects an unknown attribute or `on:` event. The most concrete missing MUST.
- ❌/🟡 **Compiler contract validation is unwired:** a *separate* string-based `ComponentContract` in the compiler is never fed from the registry, and there is **no `compile()` orchestrator** (`okui-compiler/src/lib.rs:5-9` only declares modules).
- 🟡 **UI IR references components by string `tag`, not `ComponentId`** (`ui-ir/.../ui_ir_node.rs:52`). Node ids are stable u64 (good); component reference is a name (name→id resolution has no home).
- 🟡 **Rust DSL** drops source spans (`UiIrDocument::new(root)` with empty source map, `ui-macros/.../dsl_macro.rs:101-106`) and collapses `{expr}` children to literal text (dynamic binding lost).
- 🟡 Expression forbidden-set is a lexical scan (`expression_validator.rs`) — cannot catch arbitrary function calls / direct service access (self-documented).
- 🟡 **Synthetic/generated parts modeled but never produced** (`NodeKind::Synthetic` has no allocator; `::before`/`::after`/selection/scrollbar unimplemented).
- 🟡 Missing **`tree-grid`** widget (catalog lists list/tree/table/tree-grid; only tree + table ship).
- ✅ Component trait/contract/registry, retained tree + keyed reconciliation, portals preserving logical scope, `#[derive(Contract)]` (unknown keys error), primitives (all 8), theme presets/assets, IR versioning/migrations.

---

## Area 3 — State, transactions, runtime (spec 03)

- 🟡 **§3.2 transactions built, pipeline bypasses them** (see M1). `TreeCommit`/`tree_transaction.rs` self-admit the runtime UI-op layer is "later".
- 🟡 **Only 4 of the 13 §3.4 loop stages** implemented (reconcile→style→layout→compose). Motion (7,9), render-graph/submit (11,12), input dispatch (3), notify (13) are stubs. **InvalidationRouter (stage 5) and FrameScheduler are complete but unwired** → styles/layout never skipped incrementally.
- 🔵/❌ **`UiRuntime` is a bare id holder** (M3); `runtime_{config,event,handle,state}.rs` are placeholders.
- 🔵 **§3.7 runtime lifecycle** (last-window/shutdown/critical-stop/release) and **§3.3 stale-result discard** are placeholder structs; `StaleResult` a stub.
- ⬜ **§3.5/§3.6 gaps:** single global scheduler (no per-window/visibility/animation-continuation); `Deadline` exists but unused (no budgeting).
- 🔵 Record/replay entirely stubbed.
- ✅ State primitives (signals/local-state/scoped subscriptions/generations), both transaction primitives (staged→validate→atomic commit; free rollback), task ports, scheduler internals (priorities/coalescing/backpressure/starvation), invalidation router, headless determinism stack. No correctness violations — issues are integration/coverage.

---

## Area 4 — Layout, CSS, Motion (spec 04)

**§4.13 supported-area coverage (as computed style):**

| Feature | Status | Feature | Status |
|---|---|---|---|
| cascade | ✅ | flow/flex/grid/positioning (from CSS) | ❌ (display hardcoded) |
| specificity | ✅ | typography | ❌ (no props) |
| inheritance (+inherit/initial/unset) | ✅ (7 props) | dimensions & spacing | 🟡 (only W/H; margin/padding→unsupported) |
| custom props / `var()` | ✅ | backgrounds | 🟡 (only background-color) |
| complex selectors | ✅ | opacity | ✅ |
| attribute selectors | ✅ engine / 🔵 runtime | transforms / overflow / shadows / blur / filters / cursor | ❌ |
| `:nth-*` | ✅ | logical properties | ❌ |
| media / container queries | 🔵 (parsed, gating not applied) | `calc/min/max/clamp` | ❌ (parsed, not evaluated) |
| hover/focus states | 🟡 engine / 🔵 runtime (pseudo_state always empty) | relative units (em/rem/vw/vh) | ❌ (px/%/auto only) |

- ✅ **Excluded features rejected:** `:link/:visited/:target`, `@import`, unknown at-rules → typed `Unsupported(...)` diagnostics. 🟡 float/clear/columns/etc. surface as generic `Unsupported(Property)` (indistinguishable from a typo); `display:table` leaks (accepted as keyword).
- ✅ Cascade origins/bands/source-order/runtime-override; 🔵 style scopes/modules & generated parts are data-model-only.
- **Layout:** ✅ 7 real algorithms + invalidation model; ❌ runtime never dispatches by `display`; 🟡 box-model carried by `LayoutStyle` but the style→layout bridge only supplies W/H; 🟡 observers real but no callback dispatch; ✅ virtualization.
- **Motion (§4.17):** ✅ interpolation/timeline/tracks/FLIP/drivers/reversal + CSS transition/@keyframes→native tracks; 🔵 interruption (policy-only)/phases (inert records); ❌ chaining/sync, style→motion→capability→paint pipeline; 🟡 narrow (no color/transform value type, 4 easing curves, cubic-bezier/steps dropped).

---

## Area 5 — Rendering, Text, Resources (spec 05)

- ✅ **Scene/display-list** (paint order, fills/strokes, clips, layers push/pop balance, gradients, blend/composition mapping), **renderer capabilities/fallbacks honest** (the most spec-faithful area), **typed errors**, host-provided device, offscreen/bitmap/texture targets.
- 🔵 **Window path = offscreen-render + `TextureBlitter` blit** (vello 0.9 has no render-to-surface); acceptable + documented, but it's an inherent method, not on the `Renderer` trait, and `render()` on a `Window` target still renders offscreen.
- ❌ **Text bypasses `obscur-text` entirely** (M1): `DrawText` carries a raw string; shaping is naive skrifa (charmap+advances, baseline≈origin+1em, no kerning/ligatures/complex/bidi) inside the vello adapter; CPU renders no text → two backends disagree (determinism boundary broken).
- ❌ **No real shaper/font provider** — only `MockTextShaper`/`MockFontProvider`; no harfrust/rustybuzz/swash/unicode-bidi anywhere. The obscur-text layout/selection/editing/IME are a **well-built disconnected island**.
- 🟡 **Resources:** load path + `ResourceHub` + `CacheStore` real; **decode/compile only `NullDecoder`/`NullCompiler`** (no image/SVG/font/CSS decoder). ❌ images/SVG/system-fonts absent.
- 🟡 **Trusted render extensions** gate-only (`VelloCustomPass` is a descriptor; pass execution deferred).
- 🟡 **Masks / layer effects / blur** modeled but not rasterized (`Clip::Mask` falls through; no layer transform/blur applied).
- ✅ Text editing engine (coalescing undo/redo, UTF-8-safe nav), IME lifecycle, sRGB/linear/premultiplied color, HDR discovery w/ sRGB fallback.

---

## Area 6 — Platform/Windows/Services + A11y/Testing/Diagnostics (spec 06 + 07)

- ❌ **Input not wired end-to-end** (M1) — the single most consequential gap. `PointerConverter`/`KeyboardConverter` complete + tested but have zero non-test callers.
- ⬜ **System providers:** only **dialog/shell/notification** have real adapters (Linux zenity/xdg-open/notify-send; Web equivalents). **Capture** and **network** are port + stub only. Document/privacy/power have real domain logic but only in-memory/fixed reference providers. macOS/Windows adapters are 1-line skeleton stubs (deferred, unbuildable on Linux).
- ❌ **DevTools protocol + server** (§7.6/§7.7) — 100% skeleton stubs (no DTOs, handshake, version negotiation, auth, transport, permission separation).
- 🟡/❌ **Diagnostics telemetry** (§7.9): `Diagnostic/Sink/Filter/PerformanceSpan/PerformanceBudget` real+exported; `PerformanceCounter`, `BudgetViolation`, `TelemetrySink`, `DiagnosticSubscription`, `DiagnosticBufferPolicy` are unexported skeleton stubs.
- 🟡 **Capability/permission model:** `SupportLevel`/`Capability*`/`PermissionState` real; **`PermissionDecision` and `CapabilityConstraint` (the gating logic) are stubs.**
- ❌ **A11y semantic tree computes no diffs** (`SemanticUpdate`s hand-built by mutator); ❌ headless snapshot omits **paint/accessibility/render-graph/rendered-image** stages; ❌ **§7.13 replay redaction** (passwords/secrets) not implemented.
- 🟡 Custom chrome regions / display enumeration: domain models complete, winit adapter returns `Unsupported`/empty (drag/resize/system-menu regions, `displays()`/`poll_display_events()`).
- ✅ Window lifecycle/state/constraints, WindowBackend port + winit deferred-creation bridge + lifecycle-event conversion, focus/gesture/spatial/selection/command/routing/DnD domains, a11y bridge + AccessKit converter, headless host + synthetic input, task adapters (smol/tokio).

---

## Area 7 — Plugins, roadmap/non-goals, tools (spec 08 + 09)

- 🔵 **Plugin system (§8): uniform honest bare skeleton.** All 9 crates materialize the spec-mandated module layout; every domain entity is a 6-line placeholder ("skeleton stub … not yet implemented"); zero real deps (no zip/tar/wasmtime/proc-macro). Deferral is clean (compiles, no `todo!()`, no over-claims). Deferred by user.
- ✅ **No non-goal violated:** okui only lowers to UI IR (no eval/VM); CSS never fetches `@import` (→ unsupported) and has no `url()` at all.
- ✅ **xtask** is the one genuinely real tool (4 architecture rules: module-layout, forbidden-file-name, unsafe-boundary, dependency-direction). ⬜ `obscur-devtools`, `obscur-okui-lsp`, `obscur-plugin-cli` print "not yet implemented".

---

## Deliberate deviations (documented, defensible)

- `obscur-ui-macros` is an expansion-logic crate, not `proc-macro=true`.
- CSS matches by **string** class/id/type (not interned symbols) — sidesteps interner coherence.
- Text shaping in the renderer; display list carries a raw run string (acknowledged simplification).
- Vello offscreen-render + blit (vello 0.9 has no render-to-surface).
- Widgets are headless state machines exposing `view()`, not `Component`-trait impls.
- CSS parser hand-rolled (no cssparser/selectors) — strengthens "no external types in public API".
- `taffy` inside `obscur-layout` (algorithm lib per §3.6, not a platform adapter).
- `@media`/`@container` rules lifted unconditionally (gating deferred, documented).

---

## Prioritized remediation (by leverage)

1. **Wire input end-to-end** (pointer/keyboard converters → interaction pipeline → pseudo-state → styling). Closes interactivity **and** unlocks `:hover`/`:focus`.
2. **Expand `PropertyId` + map `display`** in `layout_style_from`. Biggest CSS/visual win (flex/grid + margin/padding/font/etc. become reachable).
3. **Wire `obscur-text` into the scene** (shape via the domain, carry positioned glyphs). Fixes text determinism + real shaping/fonts.
4. **Build a real `UiRuntime`** that owns the pipeline/scheduler/transactions/invalidation. Closes the §3 integration gaps (transactions on the live path, incremental invalidation, lifecycle).
5. **CSS depth:** apply `@media`/`@container` gating, evaluate `calc()`, add relative units.
6. **Cheap MUST-fix:** reject unknown attributes/events in the okui compiler (§2.13); add a `compile()` orchestrator bridging the real registry contracts.
7. **If needed:** diagnostics/devtools protocol+server, permission-decision/capability-constraint, resource decoders (image/svg/font), a11y diffing + snapshot stages + replay redaction, harden xtask to a cargo-metadata graph.

---

*Generated from a 7-agent read-only conformance audit at commit `355cc29`. Every ❌/🟡 row is backed by `file:line` evidence in the agents' raw reports.*
