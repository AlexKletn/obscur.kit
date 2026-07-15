# obscur.kit_v2 — Spec Conformance Audit

- **Date:** 2026-07-15
- **Commit:** `f074dba`
- **Workspace:** `cargo run -p xtask -- check` clean; 64 workspace packages (59 crates + 4 tools + facade); 365 commits.
- **Method:** 8 parallel read-only audits (one per spec chapter §1–§9), each enumerating the chapter's concrete requirements and verifying them against the code at HEAD with file:line evidence. Supersedes the 2026-07-08 audit (commit `7736b19`).
- **Tiering:** **Tier-1** = actively wrong / core-required semantics missing. **Tier-2** = spec-required, real gap. **Tier-3** = deferred / cosmetic / enforcement-breadth / documented-policy (not a defect).

---

## Overall verdict

The architecture is **strongly conformant** and its boundaries hold live (machine-enforced). Rendering/text/resources (§5) and layout/CSS/motion (§4) are **genuinely wired end-to-end** — the old "built but not wired" theme is largely closed: text is shaped once and consumed identically by both backends, CSS transitions/@keyframes are now driven by the MotionScheduler, and the transaction commit boundary is live. Accessibility, testing and diagnostics (§7) advanced the most since the last audit — a real SemanticTree from the retained tree, a security-enforcing DevTools stack with live runtime-backed inspection, 8-stage headless snapshots, and record/replay with default redaction.

The **deepest cross-cutting real gap is §3: invalidation is not incremental** — the scheduler gates whole *phases* by `FrameReason`, but when a phase runs it recomputes the *entire* tree, and the per-component instance/state model (`Component`/`LocalState`/`ReadSignal`) is built but never driven by the runtime (single wholesale root rebuild). This is the root of interaction cost (the wireframe hover spike; the recent per-frame `MeasureCache` softened it but is a memo, not incrementality). The other headline deviations: §2.8's **primary proc-macro TSX DSL does not exist as a macro** (only the codegen fallback ships) and `#[derive(Contract)]` is inert; and platform breadth is thin — **macOS/Windows system providers are inert placeholders** so those targets ship zero services.

No non-goal is violated; the DevTools guardrail (§9.6 — no arbitrary-code channel) is respected by construction. The plugin tier is an honest, over-claim-free deferred skeleton.

---

## Remediation log (post-audit)

The findings below are recorded **as of commit `f074dba`** and are left unedited so the audit stays a snapshot. This log tracks what has since been fixed.

| # | Finding | Status |
|---|---|---|
| §3-1 | Invalidation not incremental | **Hot path closed.** Layout is now skipped when nothing that moves a box changed, driven by real per-node deltas through the `InvalidationRouter`. *Within-phase* recalc (style still resolves the whole tree) remains open. |
| §2-3 | Primary proc-macro TSX DSL absent | **Closed.** `obscur-ui-macros` is a real proc-macro crate exporting `okui!` + `#[derive(Contract)]`; `okui!` lowers to `View`-building Rust at expansion time (no runtime IR). `#[derive(Contract)]` is now invocable but still has no in-tree usage. |
| §4-4 | Grid drops its own min/max clamp | **Closed.** Grid clamps its content extent via the shared `clamp_content`, at parity with flow/flex. |
| §4-5 | Flex ignores `align-self` + cross min/max | **Closed.** `item_align` resolves per-item `align-self` (the same resolver grid uses); `clamp_cross` applies the item's cross bounds. |
| §2 (Tier-2) | Shipped widgets have no default contract | **Closed.** `widget_contracts()` seeds all 12 widget tags into the default registry, mirroring each widget's §2.6 builder API; facade drift guards pin the mirror. `command` is deliberately not a prop (it is the event handler); the collection widgets stay open (no string vocabulary exists to mirror); "no children" remains unexpressible by the contract type. |
| §3-2 | Component-instance / local-state model unwired | **Closed.** The runtime mounts instances (keyed by `NodeId` in a `SecondaryMap`), decodes typed props from the node's now-retained properties, calls the author's `render` behind an `ErasedComponent`, and re-renders **only** stale components via `UiOperation::Rerender` + `Reconciler::reconcile` — a pair that previously had zero non-test callers. `request_update` finally has a reader. Three real bugs surfaced en route: `#[derive(Contract)]`'s prop symbols disagreed with the view's; `RetainedNode` discarded every non-CSS prop (so a change to one produced no `Update`, which also let the layout gate wrongly skip); and the root reconcile deleted a component's output every frame (fixed by `NodeFlags::COMPONENT_HOST`). **Reactive entry point closed too**: `SignalId` + `SignalGenerations` + `ComponentContext::track` + `UiRuntime::notify_signal` implement §3.1's "dependency tracking invalidates only consumers of changed values" — a host mutates its own data and names what changed; only components whose last render read that source re-render. `Subscription` previously recorded a generation but not its source, so staleness was unanswerable; it now names the source. The root `FnMut() -> View` remains available (and unchanged) for hosts that want it — an app that stays on it still rebuilds wholesale, by its own choice. |
| §6-6 | macOS/Windows providers inert | Open (untestable from this host). |
| §7-7 | Synthetic input by selector is descriptor-only | **Closed.** `test-id` added as a reserved symbol outside `StyleAttributes` (so §7.5's "affects no CSS/layout/visuals" holds by construction); `SelectorResolver` resolves Node/TestId; `SyntheticDispatcher` replays through the runtime's REAL `handle_pointer`, so hit-test/hover/focus/command behave as for a user. A coordinate is deliberately left for the runtime to hit-test rather than resolved twice. `DispatchOutcome` reports target, propagation path (logical parents), default action and hover/focus changes. **Not** included: state-change/layout-change records (the runtime does not produce them — inventing them would be a second unverified source of truth) and key dispatch (no keycode→KeyEvent map yet). |

**Correction to §3-1's original wording.** The finding said the accumulators "are wired but never fed real node invalidations". That was right, but the first fix for it was itself dead code: it gated on `FramePlan::reconcile()`, and `run_due_frame_reactive` — the path every windowed host drives — classifies *every* frame as a structural `UiCommit`, so the refinement never ran in a real window. The gate is now the reconcile's actual result (an empty commit proves the tree byte-identical). Measured on the wireframe: paint-only hover 5.88ms → 2.63ms, `relaid_out` true → false.

---

## Per-area verdicts

| Area | Verdict | Tier-1? |
|---|---|---|
| §1 Architecture & boundaries | Strongly conformant; boundaries machine-enforced; gaps are enforcement *breadth* | none |
| §2 Components, tree & DSL | Data model faithful & wired; DSL/contract *macro* surface missing | **yes** |
| §3 State, transactions & runtime | Commit boundary now live; invalidation not incremental; component model unwired | **yes** |
| §4 Layout, CSS & motion | Strong; motion now live; MeasureCache pure; grid/flex clamp inconsistencies | **yes** |
| §5 Rendering, text & resources | Genuinely wired end-to-end; effect-split is documented policy | none |
| §6 Platform, windows & services | Ports + host-provides-device clean; macOS/Windows inert; adapters under-wired | **yes** |
| §7 A11y, testing & diagnostics | Substantially wired; strong security; selector-input & event-push gaps | **yes** |
| §8 Plugin system | Honest deferred skeleton, no over-claim | none |
| §9 Roadmap & non-goals | All non-goals respected (several actively enforced) | none |

---

## Tier-1 findings (actively wrong / core-required missing)

1. **§3 invalidation is not incremental.** `FramePlan::for_frame(reason, …)` maps each `FrameReason` to one *global* invalidation and reads back only per-phase dirty booleans; a running phase recomputes the whole tree (match/style over the whole tree, `layout_editing` over the whole tree). The per-node `StyleInvalidation`/`LayoutInvalidation` accumulators and `InvalidationRouter` node-forwarding are wired but never fed real node invalidations. Real wins are phase-*skipping* (Input skips reconcile, TextEdit is compose-only, DiagnosticsCapture idle), not subtree-incremental recalc. *(ui_runtime.rs `run_frame` ~1047; frame_plan.rs; invalidation/invalidation_router.rs)*
2. **§3 component-instance / local-state model unwired.** `Component` trait, `LocalState`, `ReadSignal`, `ComponentRegistry`, `ComponentFactory` are complete in `obscur-view` but the runtime never instantiates/renders a component or tracks dependencies — reactivity is a single root `FnMut() -> View` rebuilt wholesale each frame; only `UiOperation::ReplaceRoot` is ever committed (`Rerender`/`InvalidateStyle`/`InvalidateLayout` are dead). *(obscur-view/src/{component,state}/; ui_runtime.rs:396; transaction/ui_operation.rs)*
3. **§2.8 primary proc-macro TSX DSL absent as a macro.** `obscur-ui-macros` is a plain lib (no `proc-macro=true`, no `#[proc_macro]` fn, no `okui!`/`dsl!` call site). `DslMacro::expand`/`ContractDerive::expand` are solid logic but unusable as macros; the spec's *fallback* codegen path (`ViewCodegen`, build.rs) is the only real surface. **§2.13 `#[derive(Contract)]` is inert** (zero usages). *(frontend/obscur-ui-macros/)*
4. **§4 grid container min/max size clamp dropped.** Flow and flex clamp their own box into authored `min/max-width/height` (`clamp_content`), but grid only `constraints.constrain` — an authored `min-height`/`max-height` on a `display:grid` element is silently ignored. Cross-context inconsistency, user-visible. *(obscur-layout/src/algorithm/grid_layout.rs ~1418)*
5. **§4 flex `align-self` + per-item cross-axis min/max ignored.** Flex reads only the container's `align_items`; per-item `align-self` and cross-axis `min/max` are dropped by the algorithm (grid *does* honor per-item `align-self`/`justify-self`). *(obscur-layout/src/algorithm/flex_layout.rs ~822)*
6. **§6 macOS/Windows system providers inert.** `obscur-system-macos`/`-windows` are placeholder unit-structs with no `obscur-system` dep, no trait impl, no target-cfg; the facade only references `-linux`/`-web` for their targets — so macOS/Windows ship **zero** system providers (dialog/shell/notification). *(crates/adapters/obscur-system-{macos,windows}/; facade Cargo.toml target gating)*
7. **§7.4/§7.5 synthetic input by selector/coordinate is descriptor-only.** `SelectorTarget::{TestId,Coordinate}` + `SyntheticInput` descriptors exist, but no engine resolves a selector→node or dispatches it through the headless host; the "read half" of testing is strong but the write-half-via-selectors isn't wired. Typed dispatch results are also thin (no propagation path / explicit default action / state-change records). *(obscur-headless/src/input/; host/headless_host.rs)*

---

## Tier-2 findings (spec-required, real gap)

- **§1** — xtask lacks the §3.16/§5.8 "multiple main entities per file (without exception marker)" and "feature-matrix consistency" checks.
- **§2** — empty/loading/separator slots and named slots (§2.11/§2.12) are not parsed/lowered by the `.okui` frontend (contract carrier types exist, unused); shipped widgets (Button/Input/Select/…) have **no default contract** in `builtin_contracts` (unchecked unless the app hand-registers a surface); §2.10 has only a lexical deny-list, no positive allow-list.
- **§3** — `ProvisionalTransaction` (DnD commit/cancel) implemented but not connected to any interaction system; ~~`obscur-task` async ports exist but the runtime uses none — resource loading is synchronous, no typed-message return or generation-based staleness~~ **PARTLY CLOSED**: `set_blocking_executor` + `poll_resource_tasks` move loads off the frame thread and return `TaskResult<T>` typed messages; staleness is the hub's existing `ResourceVersion` rejection (no second generic generation was invented, and the `StaleResult` placeholder was deleted rather than given a contract). `Spawner`/`Timer` are still undriven — nothing in the runtime needs a future or a deadline yet. The frame pipeline still omits input-dispatch-as-phase and render-graph/submit/present.
- **§4** — no CSS parse cache (§4.12, `SourceId` used only for spans); logical properties missing (`margin-inline`, `inset-block-*`); intrinsic sizing is a single constrained measure (no distinct min-content/max-content pass); `ch` unit unsupported.
- ~~**§5** — SVG is decoded/validated but never painted (§5.17)~~ — **CLOSED.** `SceneVector`/`VectorPaint` keep `currentColor` deferred; `SvgVectorizer` detects it by parsing twice with different injected colours and diffing (usvg bakes it at parse time, so nothing else could tell it from black); the coordinator emits `DrawPath` under a placement transform and resolves `currentColor` from computed `color` per frame. Both renderers already consumed `DrawPath`, so neither changed. Proven e2e with the source file deleted before the theme change. Not carried: SVG gradients/patterns/masks/filters/text.
- **§6** — `WinitWindowBackend` port adapter is exercised only by its own unit tests; every windowed example bypasses the port and calls winit directly. winit `displays()`/`primary_display()` return empty (§6.6 unreachable through the port). No platform adapter for the real §6.20/§6.21 clipboard/DnD domain nor §6.17 capture; capability gating is type-level only (`PermissionDecision` defined but unexported); the §6.7 Action model drives no menu/toolbar/palette/tray; §6.10 document access is sync-only.
- **§7** — live **event push** over the socket is absent (request/reply only; `fan_out` exists but isn't streamed); inspect targets Animations/EventFlow/Resources/Caches/PluginState fall back to typed-default payloads (host registers only Tree/Layout/Styles/Focus/Rendering); no distinct `automation-id`; span attribution limited to subsystem/frame (no node/resource/invalidation/plugin attribution).

---

## Tier-3 findings (deferred / cosmetic / enforcement-breadth / documented-policy)

- **§1** — banned-external + `forbid(unsafe_code)` scans skip `crates/frontend`+`crates/components` (both "pure", currently unexploited); §1.3's granular edge bans (`renderer→runtime internals`, `plugin model→plugin host`) subsumed into coarse layer buckets; `default-members` points at `obscur-runtime` not the facade; facade is `publish=false`/`0.0.0`; recommended §5.7 clippy lints not configured.
- **§3** — §3.7 lifecycle entities are placeholder stubs (no runtime events emitted); §3.8 persistence helpers absent (optional); per-window scheduling / hidden-minimized suspend not modeled (single-surface runtime).
- **§4** — the CSS→runtime motion repackaging exists only as a test/example helper (both ends real, proven e2e; ~20 lines to promote to a library fn); `PositionedLayout` algorithm is a vestigial stub (real positioning lives in the coordinator); per-container overflow scroll not wired (single viewport scroll).
- **§5** — wide-gamut/HDR discovery, image ICC profiles, explicit window/monitor color spaces; animated/progressive images, DPI variants, nine-slice; CPU luminance/alpha *mask* clips inherit parent clip (approx). CPU effect approximations (gradients→first-stop, shadow→flat, backdrop-filter skipped) are **documented policy**, correctly reported via `supports_effect` — not defects.
- **§6** — §6.8 tray, §6.14 app-lifecycle hooks, §6.15 insets/virtual-keyboard/safe-areas, §6.16 mobile gestures, §6.18 media devices, §6.19 media playback — absent (all marked optional in the spec).
- **§7** — `PerformanceSpan` carries no duration (**deliberate**: wall-clock timing is host-side via `obscur_kit::perf::FrameProfiler`, keeping the runtime clock-free); AccessKit bridge stores the `TreeUpdate` for the host rather than owning a live platform AX handle (port pattern).
- **§8** — `obscur-plugin-abi` lib doc mentions "unsafe permitted behind guards" without a `#![forbid(unsafe_code)]` (contains no unsafe; aspirational doc while deferred).

---

## What genuinely changed since the 2026-07-08 audit

- **§3** transaction commit boundary is now **live** (`ReconcileEnactor` in `run_frame`, version-stamped/observable) — previously test-only. `InvalidationRouter` is **wired** (drives real phase-skipping) — previously orphaned. (Still phase-level, not subtree-incremental.)
- **§4** CSS `transition`/`@keyframes` are **driven** (matched → `MotionScheduler` → sampled pre-layout) — the prior "compiled but not registered" gap (S4) is closed. Min/max-**height** clamp added to flow+flex. Per-frame `MeasureCache` added (pure memoization; layout ~73→20ms on the wireframe).
- **§7** the whole DevTools **interface** was built out: `UiRuntime::inspect_snapshot` + `semantic_snapshot`/`semantic_tree`, the external `obscur-devtools` CLI, a security-enforcing endpoint over a real loopback/Unix socket, live runtime-backed `inspect_source_from`/`live_source`/`SharedInspectSource`, and frame perf (`FrameStats`/`FrameProfiler`) exposed as `Inspectable::Rendering`.
- **§2** the `.okui` **codegen** path (`ViewCodegen` + `name_binding` registry + facade `build.rs`) now compiles `.okui` to real `View`-building Rust; the runtime bridge maps tags→real `COMPONENT_ID` and `<img src>`→`DrawImage`.

---

## Highest-leverage follow-ups (my read)

1. **Incremental invalidation (§3).** Feed real per-node style/layout invalidations into the wired `InvalidationRouter` so a hover/edit recomputes only the affected subtree — the single biggest correctness-vs-spec and perf win (it, not the MeasureCache, is the real fix for interaction cost).
2. **Grid + flex clamp parity (§4, Tier-1).** Extend the `clamp_content` own-box clamp to grid, and apply per-item `align-self`/cross-axis min-max in flex — small, mechanical, removes a user-visible inconsistency.
3. **Widget default contracts (§2.13).** Seed `builtin_contracts` with the shipped widgets so `<Button>`/`<Input>` misuse is caught without manual registration.
4. **SVG paint (§5.17).** Wire a scene command that consumes the already-decoded SVG.
5. **macOS/Windows system providers (§6).** Even minimal real dialog/shell/notification adapters so those targets aren't service-less.
