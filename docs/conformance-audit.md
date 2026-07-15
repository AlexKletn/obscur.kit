# obscur.kit_v2 — Spec Conformance Audit

- **Date:** 2026-07-15
- **Commit:** `f01df9d` (+ the fixes this audit prompted; see *Fixed during the audit*)
- **Workspace:** `cargo run -p xtask -- check` clean; 63 packages; 4,629 tests passing; 412 commits.
- **Method:** 9 parallel read-only audits — one per spec chapter §1–§9, plus a cross-cutting orphan sweep. Each derived its requirements from the chapter text (quoting the line), then verified against the code at HEAD with `file:line` evidence.
- **Supersedes** the 2026-07-15 audit at `f074dba` (47 commits earlier). That audit is superseded rather than amended because its central claims were **systematically too generous**, in one specific and repeatable way — see below.
- **Tiering:** **Tier-1** = actively wrong (produces incorrect output while reporting success) or core-required missing. **Tier-2** = spec-required, real gap. **Tier-3** = deferred / documented policy / honest placeholder.

---

## Why this audit exists

The previous audit called §5 "genuinely wired end-to-end" with "none" Tier-1, and recorded the "built but not wired" theme as "largely closed". Both are false, and the *reason* they are false is the finding that matters most:

> **A type existing is not evidence that anything calls it — and a doc comment asserting the wiring is how a reviewer gets convinced otherwise.**

Three doc comments in this codebase assert an architecture that is not connected:

- `child_measurer.rs:19` — leaves "route to the intrinsic-measure port `IntrinsicMeasure`". They do not; `IntrinsicMeasure` has no production consumer.
- `layout_invalidation.rs:12` — "the structured replacement for scattered ad-hoc dirty flags". The shipped model *is* four booleans.
- `frame_budget.rs:15` — "the emitted `Diagnostic` flows through the normal diagnostics channel". Nothing emits, and there is no channel.

This audit therefore treats **"who calls it?"** as the primary question, and reports a doc that describes an unbuilt mechanism as a finding in its own right.

### The dominant pattern

**The domain crates are richer than the runtime that consumes them.** Roughly a third of `obscur-layout`, all of `obscur-system`, all of `obscur-routing`, all of `obscur-windowing`, the whole diagnostics subscription tier, the whole record/replay tier, and every §5.19 cache compile, pass tests, and are called by nothing.

The structural cause is visible in one file: **`crates/facade/obscur-kit/src/lib.rs` is a re-export barrel** (lines 30–105 are `pub use X as y;` and nothing else). Anything whose only path to the outside is that file is unwired by construction. That is what hid `obscur-data-transfer` from the last audit; it now accounts for `obscur-system` and `obscur-routing` wholesale.

---

## Fixed during the audit

Three defects the audit found in work from the immediately preceding campaign. All were mine; all are fixed at HEAD.

| Defect | Why it mattered |
|---|---|
| **§2.10 skipped body `{expr}`** | `check_expression` is a correct, total allow-list — and `validate_element` only showed it *attributes*. A body interpolation is lowered as a real binding and emitted as real Rust, so `<span>{std::fs::read(p)}</span>` reached generated code **unchecked**. The rule was never wrong; bindings were made executable and half the places they occur were left unvalidated. |
| **`finish_drag` doc over-claimed** | Said a drop's `Rerender` means "exactly the dropped-on node reconciles". `FramePlan::for_transaction` discards every `node` payload and a reconciling frame calls `commit_root` over the whole view. |
| **`for_transaction` doc over-claimed** | Said the plan "says precisely what has to re-run". It decides which *phases* run and nothing finer — the phases take no dirty set. |

---

## Per-area verdicts

| Area | Verdict | Tier-1 |
|---|---|---|
| §1 Architecture & boundaries | Static architecture real and machine-enforced; **dynamic architecture (topology, diagnostics, error boundaries) is types with no execution path** | **yes** |
| §2 Components, tree & DSL | Data model faithful; control flow now real; **§2.12 unimplemented below the type layer; §2.13 unenforced on the primary surface** | **yes** |
| §3 State, transactions & runtime | Frame skeleton, scheduler, DnD, resource staleness real; **invalidation is boolean-per-phase; §3.7 100% stubs** | **yes** |
| §4 Layout, CSS & motion | Flow/flex/grid genuinely strong; **relative units die at the layout↔paint seam; `@container` always matches; `::before` leaks** | **yes** |
| §5 Rendering, text & resources | SVG/currentColor and resource versioning excellent; **no font subsystem; caches dead; diagnostics decorative** | **yes** |
| §6 Platform, windows & services | Two live targets (linux, wasm32) with real providers; **the whole window/display/action half is unconsumed and silently lies** | **yes** |
| §7 A11y, testing & diagnostics | DevTools read-side security genuinely enforced; **a11y tree misreports state; headless determinism is ceremonial** | **yes** |
| §8 Plugin system | Honest, well-labelled, entirely unbuilt skeleton — **no over-claim** | none |
| §9 Roadmap & non-goals | All non-goals respected; **three enforced by construction**, eleven merely unviolated | none |

---

## Tier-1 findings

### §1 — the dynamic architecture has no execution path

**1-A. A component render failure keeps stale UI and is reported on no channel.**
`ui_runtime.rs:1393` — `let Ok(view) = mounted.render(..) else { errors += 1; continue; }`. That `continue` is the whole of §1.7. The subtree keeps rendering its *previous frame's output*; the boundary policy is never consulted (`error_boundary_erased()` has zero callers); no `Diagnostic` is constructed; `ErrorBoundaryPolicy::disabled()` is indistinguishable from enabled. The complete vocabulary — `ErrorBoundary`, `ComponentFailure`, `ComponentRecovery`, `SubsystemRecovery`, `RecoveryAction::{Fallback,Retried,BoundaryReplaced}` — is unreachable from a running frame. **Worse than unimplemented: the frame reports success while the UI is silently wrong.**

**1-B. No production code in this workspace ever constructs a `Diagnostic`.**
`Diagnostic` carries exactly the five §1.6 fields and is built zero times outside tests. `DiagnosticSink` has one impl, in its own crate, with no consumers. §1.6's diagnostics contract describes a struct definition, not behaviour.

**1-C. The kit cannot create a window (§1.4 R8).**
`obscur-runtime` does not depend on `obscur-windowing`. `WinitWindowBackend` (526 lines) has zero consumers; the one example that opens a window says in prose that it does not use it (`windowed_counter.rs:22-25`). Overlay mode (R10) has no representation.

### §2 — the primary authoring surface is unchecked

**2-A. §2.13 is unenforced where it matters.** Three compounding gaps: the `okui!` macro — self-declared "the spec's *primary* authoring surface" — runs **no** contract validation; the `.okui` path skips any tag with no registered contract; and `register_surface` is called only from a test, so no user component's contract can be checked. "Unknown attributes… are compile errors" holds only for the 20 hard-coded built-in tags, and only on the file path.

**2-B. Dynamic text bindings are silently wrong on both front ends.** `okui! { <div>{user.name}</div> }` renders the literal text **`"user.name"`** (`dsl_macro.rs:138`), and a test at `:551` *asserts the bug*. The `.okui` path renders an empty box. This is the exact defect commit `d1a11bc` claims to have removed — it was fixed for attributes and left live for text, the more common case. The test file is titled "bindings carry values" and every binding test it contains is an **attribute**.

**2-C. §2.12 is unimplemented below the type layer.** `ViewNode` does not model slots (one anonymous `children`); `SyntaxKind` has no slot node; `SlotContract`/`SlotCardinality`/`StylePartContract` have zero production consumers; `Template::slot_names()` and `SymbolKind::Slot` are dead. Nothing parses, validates or routes a slot.

**2-D. `<VirtualFor>` is `<For>` with a different spelling.** Parsed, classified, lowered with its tag — and `emit_loop` never reads the tag. A tag whose entire purpose is a performance contract, honoured by nothing, with no diagnostic.

### §3 — invalidation is boolean-per-phase

**3-A. §3.4 step 8 "incremental layout" does not exist.** Every node payload in the chain — `UiOperation::{Rerender,InvalidateStyle,InvalidateLayout}.node`, `Invalidation::node()`, `StyleInvalidation::invalidate(node,…)`, `LayoutInvalidation::invalidate(node,…)` — is **write-only**. No production reader of `is_invalidated`/`reason` exists. Both phase consumers (`StyleCoordinator::resolve`, `LayoutCoordinator::layout_editing`) take no dirty set and walk the whole tree. A hover on one node ⇒ full restyle of every node. §4.3 closes with *"Scattered ad hoc dirty flags are not the architectural model"*; the shipped model is `style_dirty`/`layout_dirty`/`scene_dirty`/`paint_dirty`.

**3-B. §3.1 dependency tracking is wired to nothing and defeated downstream.** `ComponentContext::track` must be called by hand and its only call site is a test. `notify_signal`/`register_component` are called only from tests; every example uses `set_root_view` (whole-view rebuild). Even if used, `run_due_frame_reactive` hardcodes `FrameReason::UiCommit` → `FramePlan::full()`, erasing `render_components`' genuine incrementality one phase later.

**3-C. §3.7 is 100% stubs.** Six placeholder structs. No runtime lifecycle, no shutdown, no release path.

### §4 — relative units die at the layout↔paint seam

**4-A. `font-size: 2rem` renders at 16px, and poisons every `em` on the node.**
`ComputedStyle` never absolutizes lengths. Layout resolves via `resolve_px(ctx)`; **paint does not** (`scene_coordinator.rs:2812,3317,3635` use `as_px()`, which honours absolute px only). Consequences: `border-width: .1em` **reserves layout space and paints nothing** — the two phases disagree; `box-shadow`/`border-radius`/`blur` in relative units become 0; `border-radius: 50%` (the circle idiom) is dead. `rem` has no chicken-and-egg excuse. **One absolutization pass at computed-value time retires ~8 silently-wrong rows.**

**4-B. `@container` always matches.** Full parse→lower→evaluate chain exists; `MediaEnvironment::with_container` has **zero callers** and there is no `container-type` property. Unset ⇒ non-restrictive ⇒ holds. §4.13 lists container queries as *Supported*. **Fails open** — the worst mode for a gate.

**4-C. `::before`/`::after` apply their declarations to the originating element.** `selector_matcher.rs:134`: `PseudoElement(_) => true`. `.badge::before { color: red }` paints `.badge` red. No `PropertyId::Content`, no generated box. Omission would fail closed; this does not.

**4-D. Intrinsic sizing is flow-`width`-only.** `is_intrinsic` has exactly one production caller. So `min-/max-width: min-content` applies **no clamp at all**; flex `flex-basis`/`width: <intrinsic>` degrades to `auto`; `grid-template-columns: min-content` **drops the entire track list**.

### §5 — no font subsystem; caches dead; diagnostics decorative

**5-A. §5.2's "explicit diagnostics" requirement is decorative.** `RendererError::UnsupportedEffect` is **never constructed**; `Renderer::supports_effect` has no production consumer. The previous audit converted this into a clean bill of health ("documented policy, correctly reported via `supports_effect`") — nothing reads it, and what it reports is wrong: CPU claims `Emulated` for blur while `DrawShadow` destructures `blur: _` and draws a hard rect. **That is `Unavailable`.**

**5-B. §5.18 has no font subsystem, and the shaper's metrics are fake.** No system font enumeration (zero fontconfig/font-kit/core-text/DirectWrite); **zero font files in the repo**, so the "bundled fallback set for WASM/isolated environments" cannot be met by construction. "Usable rendering out of the box" is false: `SceneCoordinator::new()` installs a **mock** shaper, an empty resolver and a single-run itemizer (⇒ **no bidi by default**). Worse, the only wired real mode, `HarfrustShaper::without_font()`, shapes real glyph *advances* while reporting **idealized 0.8/0.2 line metrics** — `FontRegistry` reads the real `ascent`/`descent` at registration and never plumbs them to `line_metrics_for`. Real text, fake line boxes, in every example.

**5-C. §5.19 is a complete, tested, entirely unused subsystem.** ~1,150 lines (`PathCache`/`GlyphCache`/`TextureCache`, `CacheBudget`/`EvictionPolicy`/`CacheStatistics`/`VisibilityPriority`) with zero consumers. The caches that run are unbounded `HashMap`s with no budget, eviction, metrics or test switch.

**5-D. `Clip::Mask` does the opposite thing on each backend** — CPU shows the content, Vello hides it — and **no cross-backend golden test exists** to catch it. §5.2 names clipping as a must-match property.

### §6 — the window half lies; the services half works

**6-A. The §6 window model has zero production consumers, and every windowed example reaches around it to winit directly.** `obscur-windowing` (~3,100 lines) is not a dependency of `obscur-runtime` or `obscur-headless`. The port proves that a trait compiles and is object-safe; it proves nothing about §6.1–6.6 working.

**6-B. Window operations report `native()` for work not done.** `create_window` with an owner + `ApplicationModal` returns `Ok` and yields an **unowned, modeless** window — the relation never reaches `window_attributes()`. `with_constraints` is stored and never read; winit's `min_inner_size` is never called. `set_transparent` returns `native()` while `apply_properties` ignores it. `set_presentation(Exclusive)` silently degrades to borderless and reports `native()`. `activate()` always returns `Activated` without asking the compositor, so §6.3's headline rule — focus-stealing denial is *normal*, report `Denied` — is unreachable. `emulated()`/`conditional()` exist for exactly these cases and are **dead in production**.

**6-C. §6.6 `displays()`/`primary_display()` return nothing on every backend**, while `DisplayConverter` — a complete, correct winit→domain projection — has zero call sites and no tests.

**6-D. §6.7's unified Action model is architecturally precluded.** `Command`/`CommandState` are used only in their own tests. `CommandRouter` explicitly disclaims them and routes `Shortcut → InternedSymbol`. The kit's own `Menu` uses `InternedSymbol`, not `Command`. Of the spec's seven surfaces, four don't exist, one ignores the model, and shortcuts work *by bypassing it*. §6.7's only "required" — fully custom context menus — is unmet.

### §7 — the a11y tree misreports state

**7-A. A disabled control is announced as enabled and actionable.** `semantic_builder.rs:104-127` never reads `is_disabled()`/`is_checked()`/`is_readonly()` — which exist and which **the CSS matcher already consumes**. `SemanticState::disabled` is always false, so `is_actionable()` returns **true for a disabled button**. The engine knows the truth on the CSS path and publishes the opposite on the a11y path. Compounded by §7.1 L27 being violated (no action path back in): an AT client is told a control is actionable and has no way to act.

**7-B. The headless runtime's determinism is ceremonial.** `advance_time`/`run_tasks` touch the clock and executor; `render`/`render_styled` touch neither. Two tests (`advance_time_moves_the_clock`, `run_tasks_drains_the_executor`) pass with the entire frame pipeline deleted — the test *itself* spawns the closure the host never spawns. **A chapter about testing has an untested test harness.**

**7-C. `Subscribe` and `Inject` are accepted, acknowledged, and dropped.** `serve_with_events` genuinely pushes — and has zero callers; every host uses `serve()`. **Nothing in the workspace ever constructs an `EventKind`.** A client is told `granted: true` and waits forever. `obscur-devtools inject` returns `Acknowledged`; nothing drains the queue.

---

## Tier-2 findings

- **§1.2 is entirely unrepresented; 5 of 7 version domains missing.** No API-level taxonomy in any form. Only UI IR and the DevTools protocol are versioned and wired; all crates are `0.0.0`.
- **§1.5 parallelism is 1-of-8, and the adapter is never consulted.** Only resource loading is off-thread. `Spawner`/`Timer`/`LocalSpawner`/`CancellationToken` have no runtime consumer; `KitBootstrap::default_task_spawner()` returns an object **nothing can accept**. No WASM workers.
- **§1.4 `ResourceHub` sharing is impossible, not "optional"** — owned by value, no accessor.
- **§2.15's compatibility story is 100% unwired** — `UiIrValidator`, `UiIrVersionRange`, `MigrationRegistry`, capability negotiation, newer-IR diagnostics: all built, zero consumers. `Compiler::compile` never validates the IR it just produced. And 15.5 ("independent of direct Rust type names") is contradicted by construction: `Binding` holds verbatim Rust source.
- **The one production `.okui` build path discards every diagnostic** (`build.rs:34-38` never reads `has_errors()`). The LSP *does* report them — so the checks aren't vacuous, they're just off the build path. Compounding: `showcase.okui` has zero bindings and zero control flow.
- **§2.6's command routing chain does not exist** (focused node → scope → ancestors → window → app fallback). `CommandRouter` is a shortcut map.
- **§2.14's registry is real machinery with an empty population** — every `ComponentFactory` impl is a test stub; no shipped widget registers.
- **§2.3 is missing `current_target` and `trusted`/`untrusted`** (zero hits repo-wide) and has 3 `EventPhase` variants where §2.3 names 4 stages.
- **§3.4 steps 9/11/12/13 are absent** — post-layout motion, render graph (`RenderCoordinator` is an empty stub), submit/present (host's), post-frame notifications (zero occurrences).
- **§3.3 provides 1 of 5 async facilities.** `TaskScope` claims component/window/runtime scoping while only `Runtime` is ever constructed.
- **§3.5/§3.6: backpressure and stale-skip are unreachable and wrong when reached.** `BackpressurePolicy::bounded(n)` **does not bound** (at capacity `decide` returns `DropStale` and `request` pushes anyway); the runtime uses `unbounded()` regardless. Deadlines have zero consumers. `SchedulerPriority::Background` is unreachable from `from_reason`.
- **§4.3/§4.12: `StylesheetCache` has zero production consumers** — including the live-CSS-reload demos, which are exactly its workload. §4.2's `LayoutContext`/`LayoutDependency`/`LayoutContainment`/`LayoutCache`/`PropagationResult` likewise.
- **§4.17's CSS→motion bridge exists only inside an integration test.** `set_css_motion`/`CssMotionRules::from_parts` have no production caller ⇒ **every shipped example that renders CSS is motionless**.
- **§5.4 `RenderHost`/`OverlayHost` are entirely absent** — zero occurrences of the spec-named types.
- **§5.10's `LineBreaker` port cannot be swapped** — `DefaultLineBreaker` is a hardcoded local in two files; there is no setter. Whitespace + CJK pairs is not UAX #14.
- **Real text adapters exist but are not the default, and the facade won't install them.** `KitBootstrap` *builds* `HarfrustShaper`/`FontRegistry`/`UnicodeTextItemizer` and hands them back; there is no installer (contrast `install_vector_resources`, which does install). ~25 examples hand-wire the identical stack.
- **§6.20: no platform clipboard adapter exists on any target**, and `grep -i clipboard` over the runtime returns **zero** — the editing session cannot copy or paste. `LazyDataProvider` is sync despite §6.20's "lazy **async** providers"; `obscur-system` has no async at all and no `obscur-task` dep, so §6.10's "async stream" is blocked at the manifest level.
- **§6.21: DnD effect negotiation is one-sided** — `DragEffect::Copy` is hardcoded; `DragSource::allowed_effects()` has zero production consumers. A `Move`-only source silently gets a `Copy`.
- **§6.9's named recursion hazard is unguarded** — the spec requires provider chains to *include* loop detection; only `origin()` ships and the doc delegates the safety property to the caller.
- **Capability gating is type-level only.** The whole `obscur-capability` crate is 189 lines; nothing checks a capability before acting. `PermissionDecision` is a **private, unexported 6-line stub**.
- **§7.2 Localization does not exist** — zero code, including the explicit non-Anglocentric minimum (Russian plural categories, numeral agreement, gender, case variants).
- **§7.8's diagnostics channel has no producers** — a complete, tested subscribe/filter/buffer/backpressure tier that nothing publishes into. `BudgetViolation::to_diagnostic` has zero production callers.
- **§7.14's raster golden facility cannot express a tolerance** — `ImageSnapshot` is an FNV-1a hash; the spec requires "explicit tolerances for anti-aliasing and font/backend variation". Mutually exclusive.
- **§7.5's semantic role/name selector is a 6-line stub**, and `obscur-testkit` (all stubs) is **not in `[workspace.dependencies]`** — unreachable by construction.
- **`Plugin → Domain` is permitted by xtask**, so §8.3's "never direct access to retained-tree/renderer internals" and §9's "direct renderer access for ordinary plugins" are **unenforced**. The non-goal holds only because plugin crates have no dependencies at all — the weakest possible reason. Banning the edge costs one list entry today.
- **§5.16: progressive decode, animated images, DPI variants, colour profiles absent** — the model fields for progress and DPI scale exist and are never written.

---

## Tier-3 — deferred, documented, or genuinely good

**Enforced by construction (violation is unrepresentable, not merely absent) — the strongest work in the repo:**

- **§9's `.okui` business-logic ban.** A positive allow-list over `syn::Expr`, one rule wired into both front ends. Refuses assignment, compound assignment (checked at `BinOp`, not just `Expr::Assign`), `await`/`async`, closures, macros, `unsafe`, `?`, loops and free calls. Its residual gap (method calls, one level wide) is named in a doc and pinned by a test.
- **§9's legacy-CSS ban.** The property table is a positive allow-list: `float` is **rejected and reported**, not silently ignored.
- **§9.6's no-arbitrary-code guardrail.** Closed 5-variant `CommandKind`, closed `ControlAction`/`InjectedEvent`, hand-written codec where every unknown tag is `WireError::UnknownTag`, per-variant `required_scope()`, `is_dangerous()` forcing audit. No eval/script/blob variant exists and none can be added without changing an enum.
- **§7.5's "affects no CSS/layout/visuals" claim.** Two independent locks: exclusion from `StyleAttributes::extract`, *and* `TreeElement::attribute` resolving only `id`/`type`. Disjoint `0x0000_02xx` symbol namespace. Verified.
- **§1.1/§1.2's boundary rules.** Verified against source, not just the rule: **one** external dependency (`taffy`) across 21 pure crates, confined to `algorithm/taffy/` with a domain-typed surface; zero `std::fs`/`net`/`process` leakage; no global singletons.

**Genuinely wired and correct:**

- **§5.17 SVG/`currentColor`** — exact two-parse diff detection, per-frame resolution from computed `color`, cached parse, installed by the facade, and correct on the nested-`<g color>` case.
- **§5.15 resource versioning / stale-job rejection** — a real version guard, load-bearing for the async path.
- **§6.21 internal DnD** — really wired into the production pointer path, real MIME acceptance before staging, genuine atomicity (cancel *consumes* the transaction).
- **§4's flow formatting context** — margin collapsing, §10.3.3 inline solve, `margin: 0 auto`, percentage margins, intrinsic-width probes. Flex §9.7 and grid §11 are real.
- **§7.7's DevTools read-side security** — disabled by default, loopback-only *by construction*, token auth, four independent scopes, version negotiation. Non-vacuously tested.
- **§7.6's live inspection** — 10 of 12 targets read live runtime state, proven over a real socket with resolved-geometry assertions. The 2 that don't are typed defaults, **disclosed**.

**Honest deferrals (declared, not hidden):**

- **§8 in its entirety.** 1,193 lines, zero `fn`/`impl`/`trait`/`enum`. Every one of 140 entities says *"skeleton stub … not yet implemented"*, and `README.md:47` declares it. **No sandbox is claimed** (`grep -i sandbox crates/plugin/` → 0 hits) — the one trap that would have been Tier-1, avoided. Crate-level docs do over-claim in present tense (`obscur-plugin-package`: "security-sensitive"; `obscur-plugin-abi`: "unsafe permitted behind guards" where no guard exists) — Tier-3, fix by appending "(skeleton)" as two crates already correctly do.
- **§1.8's security stance**, §5.3 Canvas, §5.20 `CacheStore` (spec-optional), §7.10/§7.11 (spec-optional), §4.5's typed spaces, `quality/*` — all labelled stubs.

**Notable documentation drift:** `README.md:47`'s "Structure-complete, behaviour-empty" is **accurate for the plugin tier and badly stale for everything else** — Phases A–C are largely done. It under-claims the toolkit, which risks the *accurate* plugin disclaimer being discounted as boilerplate.

---

## Recommended order

1. **§4-A** — one absolutization pass at computed-value time retires ~8 silently-wrong rows and fixes the layout/paint disagreement.
2. **§4-B, §4-C** — both **fail open**; the two most dangerous entries here.
3. **§7-A** — the a11y tree has the data and publishes the opposite; small fix, real users.
4. **§1-A/§1-B** — a component failure should not produce stale UI and total silence.
5. **§5-B** — plumb `FontRegistry`'s real metrics to `line_metrics_for`, and give `KitBootstrap` an installer.
6. **§6-B** — return `emulated()`/`conditional()`/`Unsupported` instead of `native()` for work not done.
7. **Wire what is already built and tested** — `StylesheetCache`, the CSS→motion bridge, `PipelineSnapshot` into `HeadlessHost`, `serve_with_events`.
8. **Ban `Plugin → render/scene/tree` in xtask** while it costs one line.

## Test-suite note

The tests are mostly good and mostly non-vacuous — they test units that nothing composes. The exceptions worth naming, because this repo's own testing chapter is §7:

- `headless_host.rs:321-336` — two tests that pass with the entire frame pipeline deleted.
- `ui_runtime.rs:6925` — comment claims the violation "flows into the diagnostics channel"; the test calls `to_diagnostic()` itself and asserts a pure function's return.
- `dsl_macro.rs:551` — asserts the `{user.name}` → `"user.name"` bug as intended behaviour.
- `frame_plan.rs:274-303` — tests `InvalidateStyle`/`InvalidateLayout` plans, which no production caller can construct.
- Contract/slot tests (`slot_contract.rs`, `style_part_contract.rs`, …) — would pass unchanged if the feature were deleted from every consumer, because there are none.
