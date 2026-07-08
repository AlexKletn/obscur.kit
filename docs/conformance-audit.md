# obscur.kit_v2 — Spec Conformance Audit

- **Date:** 2026-07-08
- **Commit:** `7736b19`
- **Workspace:** 4021 tests passing, 0 failures; `cargo run -p xtask -- check` clean; 63 workspace packages (59 crates + 4 tools); 292 commits.
- **Method:** update of the read-only per-area audit against `specs/obscur-kit-2-spec-drafts/*` + `specs/obscur-kit-2-crate-architecture/*`, cross-referenced to the change chronicle (commit-tracked closure log) with symbol/`file` spot-checks in the current tree. No code changed by this audit.
- **Status legend:** ✅ conformant & wired · 🟡 partial · 🔵 modeled/deferred by design (documented) · ⬜ port-only / stub · ❌ missing or VIOLATES spec.
- **Supersedes** the 2026-07-05 audit (`fe3ca01`, 2906 tests), which pre-dated the visual/text/interaction/UA closure programs. That report's headline theme — *"built but not wired"* — is now **largely closed**: every Tier-1/Tier-2 violation and all but the deliberately-deferred Tier-3 seams have been resolved with commit-tracked fixes.

---

## Executive verdict

Architecturally the toolkit remains **strongly spec-faithful** — the full crate catalog exists, hexagonal boundaries hold under inspection (no umbrella deps, no external types in pure layers, `taffy` quarantined, object-safe ports, `forbid(unsafe_code)` in pure crates, external effect/decoder deps confined to adapters), and the domain primitives are complete and heavily tested (+1115 tests since fe3ca01).

The decisive change since fe3ca01 is that **the runtime is now a real, integrated engine, not a scaffold.** The prior audit's central finding — complete subsystems built but never called by the live path — has been closed subsystem by subsystem. What became genuinely real and wired end-to-end (each verified through the live `UiRuntime::render_now`/`run_frame` path with concrete end-to-end tests in `crates/facade/obscur-kit/tests/`):

- **Full text pipeline** wired into render on **both backends**: shaping (harfrust adapter) → bidi/UAX-9 visual reordering → script/font itemization → coverage-based font fallback → line-break + greedy wrap → justification + hyphenation + `text-overflow` ellipsis → measurement/baseline into layout → per-segment font-tagged glyph paint. CPU rasterizes **real glyph outlines** (skrifa→tiny-skia); Vello draws native glyphs. Shaping is cached (paragraph + run level).
- **Full CSS Grid** — track sizing (`fixed`/`fr`/`auto`/`minmax`/`%`), `repeat()` incl. `auto-fill`/`auto-fit`, line placement + spanning + negative + named lines, `grid-area`, `grid-template-areas`, `grid-auto-flow` (incl. dense), implicit tracks, item + content alignment. Replaced the degenerate 1-column grid.
- **The visual property set**: borders (per-side width/style/color, **mitered triangle trapezoids**, **dashed/dotted stroke**), `border-radius` (rounded fill + clip + rings), gradients (linear/**radial**/**conic**/**repeating-linear**), `box-shadow` (drop + **inset**), `filter: drop-shadow()`, **`opacity` as a composited group layer** (CPU offscreen compositing, not passthrough), 2D `transform` (+origin), `background-image: url()` + `background-{size,position,repeat}` + **multiple background layers**, `<img>`/`object-fit`, `box-sizing`, `overflow` clipping, `outline`+`outline-offset`.
- **The own-wgpu-pass render architecture** the effect-split policy called for: `filter: blur()` and **`backdrop-filter`/acrylic** are computed by our own gaussian + composite wgpu passes on offscreen textures **over the Vello scene** (Vello 0.9 has no content-blur/color-matrix primitive), plus headless Vello→PNG readback for faithful GPU-effect goldens.
- **Events / interaction**: pointer click→command→handler→reactive re-render, keyboard shortcuts + Enter/Space activation, `EVENT_INPUT`/`EVENT_CHANGE`/activation dispatch, real text editing (caret, selection, mouse click-to-position, drag-select, undo/redo, IME composition) with the runtime owning the editing session.
- **The full interaction + structural pseudo-class set**: `:hover`/`:focus`/`:active` (interaction) + `:disabled`/`:checked` (structural), all sourced into the matcher from live state.
- **`calc()`/`min()`/`max()`/`clamp()` + relative/viewport units** (`em`/`rem`/`vw`/`vh`) and `@media`/`@container` **gating** at match time.
- **§2.13 component-contract enforcement** via the `ComponentSurface` string bridge (a real `.okui` compile now rejects unknown attrs/events/children/missing-required-props).
- **A11y semantic tree** derived from the retained tree (`UiRuntime::semantic_snapshot`), **telemetry emitted** from frame phases, **transactions enacted** (`ReconcileEnactor`), **invalidation gating** frame phases (live `InvalidationRouter` via `FramePlan`), **real DevTools socket transport** (Unix/TCP) + **real LSP JSON-RPC over stdio**, and the **UA default stylesheet at `SystemDefault` origin** (overridable base component styles for `button`/`input`/`checkbox`/`select`/… incl. focus rings and state pseudos).

**What remains is now a small, honest residue**: the plugin tier (Phase 6, deliberate skeleton), the macOS/Windows system adapters (skeleton stubs), general `filter`/`backdrop-filter` **beyond** blur/drop-shadow (color-matrix — a GPU project blocked on Vello 0.9), and a set of documented edge/advanced deferrals. These are cleanly absent, not faked.

---

## Consolidated violations & gaps — resolution ledger

The fe3ca01 Tier-1/2/3 items, each marked against the change chronicle. **Tally: all 8 V-items resolved; all 7 S-items resolved (S5/S6 fully, S7 signaled).**

### Tier 1 — actively wrong (fe3ca01) → all RESOLVED

| # | Spec | Old violation | Resolution |
|---|---|---|---|
| **V1** ✅ | §4.13 | `@media`/`@container` dropped → conditional rules always applied | `MediaEnvironment` evaluated at match time in `StyleMatcher`/`rule_condition`; conditions lowered in `obscur-css/condition_lowering.rs` and gated live (`style_matching_coordinator`, `frame_pipeline`). |
| **V2** ✅ | §5.10 / §5.2 | Text shaping bypassed `obscur-text`; CPU rendered no text → backends diverged | Text is shaped **once** in `obscur-text` (harfrust adapter) into a font-tagged `GlyphRun`/`DrawText`; **both** CPU (skrifa→tiny-skia outlines) and Vello consume the same run. Determinism boundary for geometry+text+opacity restored (per the CPU/GPU effect-split policy, effect fidelity is intentionally GPU-only). |
| **V3** ✅ | §4.17 | Motion `Blend` interruption inert → values snapped | `TransitionInterruption` enacted: `Blend` redirects from the current sampled value. |

### Tier 2 — spec-supported but rejected (fe3ca01) → all RESOLVED

| # | Spec | Old gap | Resolution |
|---|---|---|---|
| **V4** ✅ | §4.13 / §4.5 | `calc()`/`min/max/clamp` + `em/rem/vw/vh` lowered to `Unsupported` | Implemented via `LengthExpr`/`LengthContext`; longhand unitless-`0` also fixed. |
| **V5** ✅ | §2.13 | Unknown-attr/event rejection inert (disjoint unpopulated registry) | `ComponentSurface` (pure-string domain surface) emitted by `#[derive(Contract)]`; `okui-compiler` `surface_bridge` seeds built-ins + registers user surfaces → a real compile rejects unknown attr / missing required prop / disallowed child / unknown event. (Option A impossible: typed contract is FNV-symbol-keyed, no reverse string resolution — documented.) |
| **V6** ✅ | §5.15–5.18 | No real resource decoders (Null only) | New adapter `crates/adapters/obscur-resource-decode` (image / ttf-parser / usvg, `resource-decode` feature); wired via runtime-owned `ResourceLoader`+`ResourceDecoder` ports; facade injects sandbox-rooted fs loader + decoder. Loaded bytes now become usable `SceneImage`s. |
| **V7** ✅ | §7.1 | A11y tree never built from `RetainedTree` | `UiRuntime::semantic_snapshot()` derives a `SemanticSnapshot` from the retained tree (`obscur-accessibility/tree`). |
| **V8** ✅ | §6.7 | Keyboard input a stub | `CommandRouter` unified to `InternedSymbol`; `handle_key` resolves shortcuts → `dispatch_command`, and Enter/Space activate the focused node; full text-edit key mapping added. |

### Tier 3 — built-but-standalone (fe3ca01) → RESOLVED / wired

| # | Spec | Old state | Resolution |
|---|---|---|---|
| S1 ✅ | §3.4 | `InvalidationRouter` orphaned | `FramePlan` consults the live `InvalidationRouter` to gate which phases run. |
| S2 ✅ | §3.2/§3.4 | Transactions not enacted | `ReconcileEnactor` drives reconciliation as a `UiTransaction[ReplaceRoot]` through the pipeline. |
| S3 ✅ | §7.9 | Telemetry never emitted | Frame-phase telemetry emitted via `TelemetrySink`+`FrameBudget` in `ui_runtime`. |
| S4 ✅ | §4.17 | CSS `transition`/`@keyframes` compiled but not driven | Matched by selector and registered with the `MotionScheduler`. |
| S5 ✅ | §7.6 / LSP | No real transport (VecDeque/stdin/`println!`) | DevTools speaks over a real `std::net` `UnixListener`/`TcpListener` (`obscur-devtools-server/transport`); LSP is real JSON-RPC over stdio (`Content-Length` framed, hand-rolled JSON — `tools/obscur-okui-lsp`). |
| S6 ✅ | §7.4 | 4/8 snapshot stages | All 8 §7.4 stages present (image stage via CPU-backend hash). |
| S7 🟡 | §3.6 | Background/starvation band never signaled | Now signaled by the scheduler (band wired end-to-end). |

---

## Cross-cutting themes

**T1 — "Built but not wired" is essentially closed.** Every fe3ca01 standalone subsystem now has a live caller: invalidation gating, transaction enactment, telemetry emission, CSS transitions→scheduler, a11y-from-tree, keyboard/text input, real DevTools/LSP transports. The remaining "modeled but unwired" items are narrow (subgrid, some observer/virtualization scaffolding).

**T2 — feature breadth now matches the engines.** `PropertyId` has grown far past fe3ca01's 27 (box-model/flex/**full grid**/positioning/z-index/typography/backgrounds/borders/radius/shadow/filter/transform/opacity/overflow/outline/box-sizing/object-fit — grep-verified through the `COMPUTED_PROPERTIES` completeness gate so authored CSS actually reaches consumers). The fe3ca01 "narrower than the engines operating on it" concern is resolved for the common web-styling surface.

**T3 — text is feature-complete**, spanning shaping/bidi/itemization/fallback/wrap/justify/hyphenate/overflow/IME, wired into both layout and paint on both backends. This was fe3ca01's single largest gap.

**T4 — the CPU/GPU effect split is now a stated policy, not an accident.** GPU-class effects (gradient ramps, blur, blends, filters) are computed on Vello; the CPU/tiny-skia backend is the deterministic reference for **geometry + text + group opacity** and *approximates* effects by design (gradients→first-stop, shadow→flat offset). Golden determinism covers geometry/text/opacity, not effect fidelity — documented and intended.

**T5 — honest deferrals remain well-managed.** The plugin tier (9 crates) is a uniform over-claim-free skeleton (0 impls/deps). macOS/Windows system adapters are skeleton stubs (`struct …Provider;`, no impl, no deps). General color-matrix `filter`/`backdrop-filter` is cleanly dropped at lower time with `supports_effect` reporting `Unavailable`. Nothing is faked.

---

## Per-area findings

### Area A — Architecture, boundaries & cargo policies — ✅ strong
Full catalog present; resolver 3 / edition 2024 / MSRV 1.90; centralized `[workspace.dependencies]`/`[workspace.lints]`; toolchain/clippy/deny/CI present; xtask green. Boundary invariants hold live (no umbrella deps, no external types in pure layers, taffy hidden, object-safe ports, `forbid(unsafe_code)`; new external deps — harfrust/skrifa/image/usvg/wgpu-effects — confined to adapters). xtask now performs a `cargo metadata`-style layering/facade-ban/cycle check (fe3ca01's string-scan gap addressed). Residual: CI still omits nextest/trybuild/OS-matrix (unverifiable in this environment); a few depend-nothing crates not centralized (cosmetic). **Known pre-existing:** repo is not `cargo fmt --check` clean (rustfmt version drift vs HEAD — do not blanket-reformat).

### Area B — Components, tree & DSL — 🟡 (strong core; §2.13 real; one bridge caveat)
Retained tree + keyed reconciler remain excellent. **V5 resolved** (`ComponentSurface` enforcement). Area-B closure also delivered `For`/`If`→`Loop`/`Conditional` IR, a mandatory-`key` diagnostic, and a `UiIrDocument → View` bridge (`UiIrViewBridge`) so the `.okui` frontend reaches the reconciler end-to-end. **Remaining:** the §2.8 primary proc-macro DSL is still a logic crate, not a `proc-macro=true` entrypoint (deliberately deferred — incompatible with the xtask lib.rs-structure rule); the `.okui` bridge maps tags via a name-hash placeholder, so real widgets (`<Button>`/`<Input>`) and reserved attrs (`src`) authored in `.okui` don't yet reach their real `ComponentId`/`SRC` symbol (the smallest next `.okui` step — a tag→ComponentId + reserved-attr registry).

### Area C — State, transactions & runtime — ✅ keystone real; stages closed
`UiRuntime` owns the world and is scheduler-gated with a `render_now` force path. **S1/S2/S7 resolved:** invalidation gates phases (`FramePlan`), reconciliation runs as a transaction (`ReconcileEnactor`), the scheduler background band is signaled. Reactive re-render (`set_root_view`/`run_due_frame_reactive`), HiDPI device-scale, and a full input arc (pointer/keyboard/text/IME/drag) are wired. **Remaining:** per-component **local** state + effects (single root view-builder only — no per-instance state); layout still recomputes broadly per frame (boxes retained for hit-test); double-click/capture-phase and live scroll gestures not modeled.

### Area D — Layout, CSS & motion — ✅ (major closure)
`display`→algorithm dispatch, full-§4.13 selector matcher + real cascade, motion sampled into computed style. **V1/V3/V4 resolved.** **Full CSS Grid** (see verdict), **positioned/absolute/fixed/sticky** layout + `z-index` stacking, **flex-wrap** (multi-line) + nested-container auto-height cross-sizing, `box-sizing`, and the complete interaction+structural pseudo-class set are wired. CSS transitions/`@keyframes` registered with the scheduler (S4). **Remaining (documented, minor):** subgrid; `grid-template` shorthand; baseline/`stretch` content-distribution; flat z-index (no nested stacking-context isolation for opacity/transform); `letter-spacing`/`text-transform`; some paint-time relative units. See `docs/grid-remaining.md`.

### Area E — Rendering, text & resources — ✅ (scene solid; text & decoders real)
Scene/display-list model solid and deterministic; object-safe `Renderer` with `HeadlessRenderer`, tiny-skia `CpuRenderer`, and the Vello adapter (host-device, offscreen→swapchain, **plus own wgpu blur/backdrop passes** and headless PNG readback). **V2/V6 resolved:** text shaped through `obscur-text` and consumed by both backends; real decoders in `obscur-resource-decode` feed `background-image: url()` and `<img>` through the `ResourceHub`. Full visual property set painted (see verdict). **Remaining (per `docs/visual-remaining.md`):** general color-matrix `filter`/`backdrop-filter` blocked on Vello 0.9; repeating-**radial** gradient; length (non-%) gradient stops; elliptical/per-corner-elliptical `border-radius`; transformed hit-testing; `clip-path`; CPU effect fidelity is approximate **by policy** (T4).

### Area F — Platform, windows & services — 🟡 (honest ports; input now real)
`WindowBackend` port faithful with honest-capability contract; winit lifecycle/property/activation/chrome/presentation real; system-service ports object-safe with `Null*` impls; Linux (zenity/xdg-open/notify-send) + Web adapters real for dialog/shell/notification. **V8 resolved** (keyboard). **Remaining:** pointer/keyboard/IME are threaded into `UiRuntime` and driven by the windowed examples, but the `WindowBackend` port still carries no input variants (input arrives via the winit event loop directly); §6.6 display enumeration returns empty at runtime; **macOS/Windows system adapters are skeleton stubs** (`struct MacosDialogProvider;` etc., no impl/deps — not cfg-gated); no central permission/consent gate; capture/privacy/power/network/document have port + `Null*` only.

### Area G — Accessibility, testing & diagnostics — ✅ (cores real & now wired)
Diagnostics channel, devtools-server (real socket transport), telemetry (now emitted), record/replay + default irreversible redaction, and the enforced §7.7 endpoint security model are real. **V7/S3/S5/S6 resolved:** a11y snapshot derived from the tree, telemetry emitted from frame phases, real DevTools socket + LSP JSON-RPC transports, all 8 §7.4 snapshot stages. **Remaining:** testkit `SemanticSelector` is still a placeholder `struct` (synthetic input by role/name/automation-id unavailable — node-id/test-id/coordinate work); the external `obscur-devtools` CLI binary is thin; a11y `alt` text carried on `<img>` but not yet surfaced into the semantic tree.

### Area H — Plugins & roadmap/non-goals — 🔵 clean deferral, no violations
Plugin tier (9 crates: abi/api/executor-process/executor-wasm/host/macros/model/package/sdk) is a uniform spec-shaped skeleton — placeholder structs, no impls/traits/`todo!()`/external deps. Nothing outside `crates/plugin/` depends on it; no non-goal violated (no native sandbox, no plugin renderer access, no DevTools code-execution channel; `.okui` executes no logic). Phase 6 is genuinely deferred, not silently half-built.

---

## Current remaining gaps & recommendations (honest residue)

1. **Plugin tier (Phase 6)** — deliberate skeleton; implement when the phase opens. No action needed now.
2. **macOS/Windows system adapters** — skeleton stubs; either implement per-family (dialog/shell/notification, mirroring Linux) or keep as explicitly-marked stubs. Only Linux + Web are real, for 3 of 8 service families.
3. **General `filter` / `backdrop-filter` beyond blur/drop-shadow** — color-matrix (`brightness`/`contrast`/`saturate`/`grayscale`/`invert`/`sepia`/`hue-rotate`) is blocked on Vello 0.9's Scene API; needs custom offscreen render-to-texture + color-matrix wgpu shaders (a GPU project) or a newer Vello. Currently dropped at lower time with honest `supports_effect` reporting. Do **not** build model-only scaffolding or CPU-compute filters (violates the effect-split policy).
4. **repeating-radial-gradient** — repeating-linear is done; radial deferred.
5. **Typography edges** — `letter-spacing`, `text-transform`, `text-shadow` (emboss), and paint-time relative units not modeled.
6. **Real scrollbars / live scroll** — `overflow` clips but there is no scrollbar UI or live scroll-gesture wiring (viewport is the only scroll container; sticky/scroll are test-drivable only).
7. **testkit `SemanticSelector`** — placeholder; role/name/automation-id synthetic input unavailable.
8. **DevTools/LSP external binaries** — the socket transport + JSON-RPC cores are real; the standalone `obscur-devtools` CLI and LSP server binaries are thin wrappers.
9. **`.okui` bridge fidelity** — tag→real-`ComponentId` + reserved-attr (`src`) mapping so authored `.okui` widgets and `<img>` render (smallest next `.okui` step).
10. **Per-component local state & effects** — the runtime drives a single root view-builder; no per-instance state/effect model yet.
11. **Grid/stacking edges** — subgrid, `grid-template` shorthand, baseline/`stretch` distribution; flat z-index (no nested stacking-context isolation for opacity/transform groups); transformed hit-testing.
12. **Known follow-ups flagged by the demos** — a suspected top/bottom border-side swap for degenerate 0-height boxes on the computed-style path (border-triangle direction, worked around in the wireframe example, not yet root-caused); vertical auto-height fill under nested flex/grid; `.ttc`/CJK fallback face selection is best-effort.

Full item-level deferral lists live in `docs/grid-remaining.md` and `docs/visual-remaining.md`.
