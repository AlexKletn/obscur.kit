# obscur.kit_v2 — Spec Conformance Audit

- **Date:** 2026-07-05
- **Commit:** `fe3ca01`
- **Workspace:** 2906 tests passing, 0 failures; `cargo run -p xtask -- check` clean; 55 crates + 4 tools; 166 commits.
- **Method:** read-only audit by 8 parallel agents, one per spec area, each mapping
  `specs/obscur-kit-2-spec-drafts/*` + `specs/obscur-kit-2-crate-architecture/*`
  requirements to the implementation with `file:line` evidence. No code changed. Supersedes the 2026-07-04 audit (`355cc29`), which pre-dated the Phase 0–5 closure.
- **Status legend:** ✅ conformant & wired · 🟡 partial · 🔵 modeled/deferred by design (documented) · ⬜ port-only / stub · ❌ missing or VIOLATES spec.

---

## Executive verdict

Architecturally the toolkit remains **strongly spec-faithful**: the full crate catalog exists (55/55), hexagonal boundaries hold under direct inspection (no umbrella deps, no external types in pure layers, `taffy` quarantined, object-safe ports, `forbid(unsafe_code)` in pure crates), and the domain primitives are complete and heavily tested.

Since the 2026-07-04 audit, the **Phase 0–5 closure delivered real, verifiable integration progress** — the single biggest theme of the last audit ("built but not wired") is materially reduced:

- `UiRuntime` is now a genuine integration keystone (owns tree/scene/coordinators/scheduler/motion/pseudo-states), not a bare id holder.
- Frame scheduling is real and gating (`run_due_frame` consults `FrameScheduler`).
- A real CSS **selector-matching + cascade engine** exists with full §4.13 combinator/attribute/`:nth-*` coverage, wired to the retained tree.
- `PropertyId` grew 7 → **27** variants; `display` drives real flex/grid/flow selection from CSS.
- **Pointer input** flows end-to-end (winit → converter → hit-test → `:hover` restyle).
- **Motion** is truly sampled into `ComputedStyles` before layout/paint each frame.
- Phase-5 cores (devtools protocol/server, telemetry, record/replay + redaction, LSP) are real, well-tested code with an enforced §7.7 security model.

**However, a residual set of concrete spec violations and unclosed integration seams remain.** They fall into three buckets: (1) a handful of features that are **actively wrong** (produce incorrect output), (2) features **rejected/inert** that the spec lists as supported, and (3) complete subsystems that remain **standalone** (built, tested, but never called by the live runtime).

---

## Consolidated violations & gaps (prioritized)

### Tier 1 — actively wrong (produces incorrect results)

| # | Spec | Violation | Evidence |
|---|---|---|---|
| **V1** | §4.13 | **`@media`/`@container` conditions are silently dropped, so conditional rules apply at every viewport.** Nested rules are lowered into the matchable set unconditionally; nothing evaluates `conditional_groups` against an environment. A `@media (max-width:1px)` rule matches always. | `obscur-css .../stylesheet_compiler.rs:338-351` (+doc :126-134); runtime matches only `compiled.matchable` (`counter_headless.rs:97`) |
| **V2** | §5.10 / §5.2 | **Text shaping bypasses `obscur-text` and is re-implemented naïvely in the Vello adapter** (`charmap.map(ch)` + advance, one glyph/char, no shaping/bidi/kerning/fallback); the **CPU backend renders no text at all**, so backends structurally diverge on an entire content class (determinism boundary broken; golden CPU tests can't cover text). | `scene_coordinator.rs:46-48,123`; `render-vello .../scene_converter.rs:156-204`; `render-cpu .../cpu_renderer.rs:234-236` |
| **V3** | §4.17 | **Motion interruption policy is inert.** Default `Blend` promises a seamless redirect from the current value, but re-animating a target hard-removes the prior timeline → the value snaps instead of blending. | policy `transition_interruption.rs:22-27`; `ui_runtime.rs:283-290` (hard `remove`, no policy consult) |

### Tier 2 — spec says supported, runtime rejects or ignores

| # | Spec | Gap | Evidence |
|---|---|---|---|
| **V4** | §4.13 / §4.5 | `calc()`/`min()`/`max()`/`clamp()` and relative/viewport units (`rem`/`em`/`vw`/`vh`, logical) lower to `Unsupported` and are dropped; only `px`/`%` work. | `property_compiler.rs:398-406,426-428` |
| **V5** | §2.13 | **Unknown-attribute/event rejection is inert end-to-end.** `okui-compiler` validates against its *own* string `ContractRegistry`, disjoint from `obscur-view`'s typed `ComponentContract` and `#[derive(Contract)]` output, and nothing populates it in a real compile → a real compile rejects nothing. The one "real violation" the prior audit flagged as *fixed* is only unit-test-real. | `okui-compiler` has no `obscur-view` dep; `validation/contract_validator.rs:48-55,229-232`; `compile.rs:305-323` skips unregistered tags |
| **V6** | §5.15–5.18 | **No real resource decoders exist.** Only `NullDecoder`/`NullCompiler`; no image/SVG/font-decode dependency anywhere → loaded bytes can never become a usable image/font/stylesheet. Loaders (`FsResourceLoader`) and caches (`FsCacheStore`) are real. | `resource_decoder.rs:100`; grep: no `image`/`resvg`/`usvg`/`png` dep |
| **V7** | §7.1 | **A11y semantic tree is never built from the retained UI tree.** The a11y domain + AccessKit bridge are real but exercised only by their own unit tests with hand-built trees; nothing derives a `SemanticSnapshot` from `RetainedTree`. §7.1's opening requirement is unmet. | grep: `SemanticSnapshot`/`SemanticTree` unused outside a11y crates; `accesskit_bridge.rs` unreferenced by runtime |
| **V8** | §6.7 | **Keyboard input is a stub end-to-end.** `KeyboardConverter` has zero non-doc callers; `UiRuntime::handle_key` routes focus + requests a frame but performs no command translation. | `ui_runtime.rs:550-556` |

### Tier 3 — built & tested but standalone (never called by the live path)

These are correct code that no live path invokes; they contradict their own doc-comments but do nothing *wrong*.

| # | Spec | Subsystem | Evidence |
|---|---|---|---|
| S1 | §3.4 ph.5 | **`InvalidationRouter` orphaned** → invalidation is full-rebuild-every-frame, not incremental; all phases run unconditionally each frame. | `invalidation_router.rs:109` (no non-test caller); `ui_runtime.rs:217-259` |
| S2 | §3.2 / §3.4 ph.4 | **Transactions not enacted.** `UiTransaction`/`TransactionCommit` are complete + tested but the pipeline mutates via `reconcile_root(&mut tree, desired)` directly; atomic-commit-between-frames is only unit-tested. | `transaction_commit.rs:9-15`; `frame_pipeline.rs:96` |
| S3 | §7.9 | **Telemetry never emitted.** `PerformanceCounter`/`BudgetViolation`/`TelemetrySink` exist but no subsystem calls `record_span`/`record_counter`. | grep: telemetry used only inside `obscur-diagnostics` |
| S4 | §4.17 | **CSS `transition`/`@keyframes` compiled but not driven.** `CompiledTransition`/`CompiledKeyframes` are produced but never registered with `MotionScheduler`. | `transition_compiler`/`keyframe_compiler`; no consumer in application/facade |
| S5 | §7.6 / LSP | **No real transport.** DevTools speaks over an in-process `VecDeque` loopback (no socket/IPC/serde); LSP is a single-shot stdin checker (no JSON-RPC); external `obscur-devtools` binary is a `println!` stub; `InspectSource` has only `NullInspectSource`. Cores are sound but unreachable by a running app. | `local_transport.rs:53-57`; `okui-lsp/main.rs:1-56`; `obscur-devtools/src/main.rs:1-6` |
| S6 | §7.4 | **Headless snapshots cover 4/8 stages** (tree/style/layout/scene); paint/a11y/render-graph/image absent. | `snapshot_stage.rs:24-35` |
| S7 | §3.6 | Scheduler background/starvation band wired but never signaled (`note_background_pending`/`service_background` have no callers). | `frame_scheduler.rs:162-197` |

---

## Cross-cutting themes

**T1 — "Built but not wired" is much reduced but not gone.** Now wired: pointer input, motion sampling, selector matching, cascade, pseudo-states (HOVER/FOCUS), `display`→algorithm, scheduler gating. Still standalone: invalidation, transactions, telemetry emission, CSS transitions→scheduler, a11y-from-tree, devtools/LSP transports, keyboard input.

**T2 — feature breadth still narrower than the engines operating on it.** `PropertyId`=27 (box-model/flex/color/opacity/visibility/display/font-size) — no transforms, shadows, filters, backgrounds beyond `background-color`, cursor, logical properties, or typography past `font-size`. Grid degenerates to a fixed 1-column uniform grid. `calc()`/units/`@media` unsupported. Positioned/docking/constraint/custom layout and observers/virtualization are modeled but unwired.

**T3 — text-pipeline correctness is the largest single-subsystem gap.** `obscur-text` (shaping/layout/line-break/bidi/IME) is a complete domain used nowhere in the render path; rendering does naïve per-char mapping in one backend and nothing in the other.

**T4 — honest deferrals are well-managed.** The plugin tier (9 crates, ~131 files) is a uniform, over-claim-free skeleton (0 impls, 0 external deps). Resource decoders, macOS/Windows adapters, and real transports are cleanly absent rather than faked. The honest-capability contract (return `Unsupported`/`Unavailable`/empty, never fabricate) holds across windowing and system services.

**T5 — audit-premise correction.** The macOS/Windows system adapters are **not** OS-cfg-gated deferrals — they are inert 6-line placeholders with no `impl` and no `#[cfg(target_os=…)]`; they compile everywhere precisely because they are empty. Only Linux and Web are real system adapters, and only for 3 of 8 service families (dialog/shell/notification).

---

## Per-area findings

### Area A — Architecture, boundaries & cargo policies — ✅ strong
Full catalog present; resolver 3 / edition 2024 / MSRV 1.90; centralized `[workspace.dependencies]` + `[workspace.lints]`; `rust-toolchain.toml`/`clippy.toml`/`deny.toml`/CI all exist; xtask green. Boundary invariants verified live (no umbrella deps, no external types in pure layers, taffy hidden, object-safe ports, `forbid(unsafe_code)`). **No violations.** Residual: xtask uses string-scans not a `cargo metadata` graph (§5.9 — no cycle/facade-dep/feature-matrix checks); CI omits nextest/trybuild/snapshots/fuzz/Miri/OS-matrix; `obscur-testkit`, plugin crates, `system-{macos,windows}`, `tools/devtools*` not yet centralized; no top-level `examples/benches/fuzz`.

### Area B — Components, tree & DSL — 🟡 (strong core, inert validation)
Retained tree + keyed reconciler are excellent (identity preserved across reorders, leaf-first removal, in-place text/attr update on stable generational ids); `obscur-view` model and CLASS/ID/TYPE→attributes path are faithful. **Violations:** V5 (§2.13 inert); §2.8 the "primary" proc-macro DSL is not a proc-macro (no entrypoint, no dependents — expansion logic exists but is uninvocable); §2.11 mandatory `key` on `<For>` is parsed but neither enforced nor lowered (Conditional/Loop IR kinds never produced; `UiIrNode` has no key field). Integration seam: no `UiIrDocument → View` bridge, so the frontend IR and the reconciler are not connected end-to-end.

### Area C — State, transactions & runtime — ✅ keystone real; ❌ two stages unclosed
`UiRuntime` genuinely owns the world and is scheduler-gated with a `render_now` force path; scheduler coalescing/priority/backpressure/starvation all implemented and tested; pointer/hover/focus/pseudo + motion sampling wired. **Gaps (S1/S2/S7):** invalidation is non-incremental (`InvalidationRouter` dead in live path — prior M1 not resolved); transactions not enacted by the pipeline; background/starvation band never signaled. Layout re-runs the whole tree every frame (boxes retained only for hit-testing). Keyboard/IME and lifecycle events unwired.

### Area D — Layout, CSS & motion — 🟡 (major closure + real violations)
Genuinely closed since last audit: 27-variant `PropertyId`, `display`→algorithm dispatch, full-§4.13 selector matcher with backtracking + specificity, real cascade (origin/specificity/importance/source-order, runtime-override beats `!important`), and motion truly sampled into computed style before layout/paint. **Violations V1/V3/V4** (@media ignored; interruption inert; calc/units rejected). Limited: grid degenerate 1-column; `[data-*]`/`[aria-*]` never match (only `id`/`type` retained); interaction pseudo-classes past `:hover`/`:focus` have no runtime source; CSS transitions/keyframes compiled but never registered (S4); positioned/docking/constraint/custom + observers + virtualization modeled but unwired; Taffy shims real but unused (native algorithms used instead).

### Area E — Rendering, text & resources — 🟡 (scene solid; text & decoders broken)
Scene/display-list model is solid and deterministic (ordered immutable `Vec<PaintCommand>`, version-gated snapshots, balance invariant); object-safe `Renderer` with real `HeadlessRenderer`, tiny-skia `CpuRenderer`, and vello adapter (host-provides-device, offscreen→swapchain blit). `ResourceHub` state machine + `FsResourceLoader` (with sandbox-root guard) + `FsCacheStore` (versioned, atomic-rename) are real. **Violations V2** (text bypasses obscur-text; cross-backend divergence) **and V6** (no real decoders). CPU gradients flatten to first stop; layers are passthrough (no offscreen compositing). Only `MockTextShaper`/`MockFontProvider` exist.

### Area F — Platform, windows & services — 🟡 (honest ports; input partial)
`WindowBackend` port is faithful with an honest-capability contract and real winit lifecycle/property/activation/chrome/presentation ops; `NullWindowBackend` reference impl. System-service ports are all real object-safe traits with `Null*` impls; Linux (zenity/xdg-open/notify-send) + Web (wasm) adapters real for dialog/shell/notification. **Gaps:** pointer input is wired only in the `windowed_counter` example, which **bypasses the `WindowBackend` port** (the port carries no input variants); **V8** keyboard stub; §6.6 display enumeration returns empty/None at runtime; **T5** macOS/Windows are inert placeholders (not cfg-gated); no central permission/consent gate (capability is per-provider self-report); §6.9 dialog loop-detection guard absent; capture/privacy/power/network/document have port + `Null*` only.

### Area G — Accessibility, testing & diagnostics — 🟡 (real cores, standalone)
Diagnostics channel, devtools-server, telemetry, record/replay + default irreversible redaction are real, well-tested working code, and the §7.7 endpoint security model (disabled-by-default, token→scope auth, version negotiation, per-command scope checks, message/subscription limits, audit log, production control-exclusion) is genuinely enforced. Headless runs the real pipeline deterministically. **Gaps V7** (a11y not linked to tree), **S3** (telemetry never emitted), **S5** (no real transports — VecDeque/stdin/`println!` stubs; `NullInspectSource` only), **S6** (4/8 snapshot stages); testkit `SemanticSelector` is an unimplemented stub, so synthetic input by role/name/automation-id is unavailable (node-id/test-id/coordinate only).

### Area H — Plugins & roadmap/non-goals — 🔵 clean deferral, no violations
Plugin tier (9 crates, ~131 files) is a uniform spec-shaped skeleton: 140 placeholder structs, **0 impls / 0 pub fns / 0 traits / 0 `todo!()` / 0 external deps**. The 9-crate decomposition maps cleanly onto §8's responsibility areas. Nothing outside `crates/plugin/` depends on any plugin crate; nothing over-claims. **No non-goal is violated** — no native sandbox is implemented or advertised, no plugin renderer access, no DevTools code-execution channel, `.okui` executes no logic. Phase 6 is genuinely deferred (not silently half-built). CLI + both executors + sdk/macros are near-empty (⬜).

---

## Prioritized recommendations

1. **Fix the Tier-1 wrongness first** (small, high-value): gate `@media`/`@container` at match time (V1); apply the motion interruption policy / blend-from-current (V3). These change correct-looking output from wrong to right.
2. **Wire text through `obscur-text`** (V2): shape in the domain, carry positioned glyphs in `DrawText`, and give the CPU backend a text path — restores the cross-backend determinism boundary.
3. **Make §2.13 real** (V5): give `okui-compiler` a dependency on `obscur-view`'s typed contracts (or populate the registry from `#[derive(Contract)]`) so unknown attrs/events actually fail a real compile.
4. **Close the two runtime stages** (S1/S2): route reconciliation through transactions and consult `InvalidationRouter` so frames become incremental + transactional (the modules already exist and are tested).
5. **Link a11y to the tree** (V7) and **emit telemetry** from the pipeline (S3) — both are "connect an existing core," not new subsystems.
6. **Breadth** (V4 + T2): add `calc()`/relative-and-viewport units, then expand `PropertyId` toward the §4.13 supported set (transforms/shadows/backgrounds/logical) and a real grid.
7. **Transports** (S5) and **remaining snapshot stages** (S6) when tooling maturity warrants; keyboard input path (V8).
8. **xtask/CI hardening** (Area A): `cargo metadata` dependency graph + cycle/facade checks; nextest/trybuild in CI.
9. **Housekeeping:** either cfg-gate the macOS/Windows adapters or keep them as explicitly-marked empty stubs (T5); centralize the remaining crates in `[workspace.dependencies]`.
