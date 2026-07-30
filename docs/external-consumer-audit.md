# obscur.kit_v2 — External Consumer Audit

- **Date:** 2026-07-30. **Audited at:** `c9f7368`. **Toolchain:** 1.96.1.
- **Vantage point — the thing that makes this audit different.** Every finding
  below was produced from **outside the workspace**: a separate Cargo project
  (`../test-app`) that depends on `obscur-kit` and nothing else, exactly as
  `docs/api/README.md` instructs an application to. That vantage is one the
  in-repo audits structurally cannot occupy — an example living in
  `crates/facade/obscur-kit/examples/` has every internal crate in its
  dependency graph, so a whole class of defect is invisible from there. Two
  Tier-1 findings below are of exactly that class.
- **The facade has no consumer inside this repository.** Check:
  `grep -rl "obscur-kit =" --include=Cargo.toml .` returns nothing; the only
  mention of the crate anywhere in a manifest is its own, plus the workspace
  member list. Everything that exercises the kit — 53 examples, every test —
  lives *inside* `obscur-kit` and therefore never crosses the boundary the
  facade exists to define. That is the structural reason finding 1-A survived
  to now, and it will keep hiding this class of defect until one consumer
  crate exists outside the facade.
- **Method.** Build a minimal application against the facade; where behaviour
  was in question, measure it rather than reason about it (a headless
  `UiRuntime::inspect_snapshot` harness prints real layout rectangles); then
  check each observation against `specs/` before calling it a defect. Several
  candidate findings did not survive that last step and are recorded as
  withdrawn.
- **Not done here.** The workspace test suite was not run; no claim below rests
  on test counts. Findings are evidenced by `file:line`, by a compiler error, or
  by measured numbers.
- **Tiering** follows `conformance-audit.md`: **Tier-1** = actively wrong or
  blocking. **Tier-2** = spec-required, real gap. **Tier-3** = documentation
  defect.
- This document decays as the code moves under it, like any other audit.

---

## Status — what this audit produced

Closed by the campaign of 2026-07-30, and this table is the ledger: a finding is
"closed" only where an implementation, a test and a green suite exist. What each
one left behind is stated under the finding itself, not rounded away here.

| # | Status | Commit |
|---|---|---|
| 1-A | closed | `191d79e` — the emitted root is an argument (`CodegenRoot`), and `checks/facade-consumer` is the consumer this repository lacked |
| 1-B | closed | `4455624` — a percentage resolves against the containing block, both writing axes |
| 1-C | closed | `86e90dc` (runtime: the model is driven) + `087a3cf` (facade: it reaches an application) |
| 2-A | closed for values | `087a3cf` — the compiled IR is kept and re-lowered per frame; structure stays static |
| 2-B | closed | `087a3cf` — `App::on_command_at` names its target by §7.5 `test-id`/`automation-id` |
| 2-C | half closed | `9f4b235` — checked on the file front end; the `okui!` half is a residue *by construction* |
| 2-D | closed | `087a3cf` — `AppControl::set_stylesheets`, over identified versioned sources |
| 2-E | closed | `9f4b235` — `ChildPolicy` distinguishes "none" from "anything" |
| 3-A…3-D | closed | `e238817` — and the rest of the drift the page had accumulated |
| `<button>` | closed | `445b6bd` — the owner's request, plus the local name every element was missing |

The line numbers this document cites are the ones it was written against
(`c9f7368`); after the campaign most of them have moved. The file:line evidence is
kept as written rather than refreshed, because an audit that edits its own evidence
to match the fix stops being a record of anything.

---

## Why this audit exists

`docs/api/12-reality.md` is an unusually honest gap list, and this document is
deliberately scoped to what that page does **not** cover. Nothing below repeats
one of its entries.

The organising observation is a companion to the one that drives
`conformance-audit.md`. That audit found that *a type existing is not evidence
that anything calls it*. This one found the outward-facing version:

> **Working inside the workspace is not evidence that anything works outside
> it.** The facade's whole purpose is to be the boundary an application sees,
> and no in-repo consumer stands on the far side of it.

The second recurring pattern is narrower and cheap to fix:

> **Several capabilities are implemented, spec-conformant, and reachable in
> `UiRuntime` — but `App` does not expose them.** The runtime is not at fault in
> any of these; the standalone entry point is.

The third is the most serious, and is this document's counterpart to the
"who calls it?" question:

> **A spec-mandated model can exist, be exported through the prelude, be covered
> by a spec test — and still be bypassed entirely by the code that runs.** §2.3's
> event model is the clearest instance: what an application actually receives is
> the one mechanism that section rules out by name.

---

## Tier-1 — actively wrong or blocking

### 1-A. `okui!` does not compile outside the workspace ⛔

**Closed in `191d79e`.** The emitted root became an argument — `CodegenRoot::facade()`
for anything an application compiles, `internal()` for a crate that really does
depend on both. The fix directions below were right about the first; the third —
re-exporting under the expected names — could not have worked, because a
`::`-prefixed path resolves in the extern prelude, which no re-export reaches.
The structural half is closed too: `checks/facade-consumer` is a crate outside the
workspace whose only dependency is the facade, built by `cargo run -p xtask --
consumer` and in CI.

The primary authoring surface (`docs/api/09-authoring-formats.md:11`) is
unusable by any crate that depends on the facade alone. First use:

```
error[E0433]: cannot find `obscur_view` in the crate root
error[E0433]: cannot find `obscur_identity` in the crate root
```

**Cause.** `ViewCodegen` emits absolute paths to internal crates —
`::obscur_view::view::…`, `::obscur_identity::…` — as string literals with no
configurable root: `codegen/view_codegen.rs:133`, `:529`, `:541`. The facade
re-exports those crates under *different* names (`obscur_kit::view`,
`obscur_kit::identity`), so the emitted paths resolve to nothing.

**Why it is invisible in-repo.** Every example lives in `obscur-kit`, which has
both crates as direct dependencies.

**Spec.** §1.2 classifies `internal` APIs as "not part of compatibility
guarantees"; the crate architecture defines the facade as "stable prelude;
curated reexports" and states it is "никогда не используется как dependency
внутренних крейтов". Forcing an application onto internal crates defeats the
stated purpose of the facade.

**Only workaround today** — and it is not a fix:

```toml
obscur-view = { path = "…/crates/domain/obscur-view" }
obscur-identity = { path = "…/crates/foundation/obscur-identity" }
```

Both are `publish = false`, so an application built this way can never be
published, and it must hard-code the workspace's internal directory layout.

**Fix directions.** Emit through the facade (`::obscur_kit::view::…`); or give
the codegen a configurable crate root; or re-export from `obscur-kit` under the
exact names the expansion already expects.

**Same defect, second surface.** The `build.rs` path documented at
`09-authoring-formats.md:41` emits the same literals from the same code and
fails the same way. Verified by reading; not separately reproduced.

### 1-B. Percentage block-size resolves against remaining space, not the containing block

**Closed in `4455624`.** The offer and the percentage basis were one value; they are
now separate, on both writing axes, and the overflow css-sizing-3 permits is no
longer clamped away. Still open, and deliberately: a percentage against an
*indefinite* containing block behaves exactly as before — that is
`12-reality.md`'s own entry, it needs the constraint model to carry definiteness,
and a test pins it so the definite fix cannot quietly move it.

In block flow, the **first** child resolves a percentage height correctly and
every subsequent sibling resolves it against what is left over, compounding down
the sibling chain.

Measured at viewport 480×240, container `.app { width: 100%; height: 100% }`,
two block children:

| Siblings | Per CSS | Actual | Consistent with |
|---|---|---|---|
| 50% / 50% | 120 / 120 | 120 / **60** | 50% of 120 remaining |
| 30% / 30% | 72 / 72 | 72 / **50** | 30% of 168 remaining |
| 20% / 80% | 48 / 192 | 48 / **154** | 80% of 192 remaining |
| 100% / 100% | 240 / 240 | 240 / **0** | 100% of 0 remaining |

**Cause.** `algorithm/flow_layout.rs:307` computes the remaining block extent and
`:319` hands it to the child as its available size; the percentage then resolves
against that constraint (`:415`, `:515`). The mirrored path for the other writing
axis is `:565` / `:575`.

**Spec.** `specs/w3c/css-sizing-3.txt:1027`, vendored by this repository:

> Otherwise, the percentage is resolved against the containing block's size.
> (The containing block's size is not re-resolved based on the resulting size of
> the box; the contents might thus **overflow** or underflow the containing
> block).

The kit is wrong twice over: it uses the wrong basis, and it suppresses the
overflow the specification explicitly permits. §4.12–4.13 adopt CSS as the
styling language and list flow layout as supported, so this is in scope.

**Why it reads as "percentages are broken".** A second sibling at `height: 100%`
computes to zero and disappears, with no diagnostic.

**Relation to known defects.** `12-reality.md` lists three layout defects — flex
border-box clamping, indefinite axes and percentages, and the missing
min/max-content pair. This is none of them: the containing block here is
**definite** (240px), and the compounding-across-siblings symptom is not
described.

**Inline axis is correct.** Block-flow siblings do not consume inline space, so
`width: 100%` on both children yields full width for both.

### 1-C. The §2.3 event model is exported, tested, and called by nothing; handlers receive nothing

**Closed in `86e90dc` + `087a3cf`.** The model is driven (capture → target → bubble →
default action, through the domain dispatcher that had no caller) and it reaches an
application: `App::on_command` takes `impl IntoCommandHandler<M>`, so a handler may
take `&mut UiEvent` and read the target, the phase, the pointer payload §2.4
specifies, and the trust flag — and may stop propagation or prevent the default
action. The fix direction guessed below is what was built. Residue: the three
bespoke signatures still bypass the walk, so a row click carries no event and no
ancestor can observe it.

This is the finding with the widest effect on how an application has to be
written, so it is stated at length.

**What §2.3 requires.** Propagation in four phases — capture, target, bubble,
default action. An event API carrying `target`, `current_target`,
`stop_propagation`, `stop_immediate_propagation`, `prevent_default`, and
trusted/untrusted origin. And a closing sentence that is unusually direct:

> Events are typed. **String-based method dispatch is not used.**

§2.4 adds the payload a pointer event is required to carry: pointer ID, device
type, buttons, logical position and delta, pressure, tilt, rotation,
primary-pointer status, hover capability.

**What exists.** All of it, as domain types in `obscur-interaction`:
`event/event_phase.rs` (`Capture` / `Bubble`), `event/event_propagation.rs`,
`event/event_dispatcher.rs`, `event/ui_event.rs`, with a spec test at
`tests/spec_ch2_interaction.rs`.

**Who calls it.** Nothing. `EventDispatcher` and `UiEvent` occur outside their
own crate in exactly two places — the facade's prelude re-export
(`prelude/prelude.rs`) and that spec test. `obscur-runtime` declares the
dependency (`Cargo.toml:28`) and then never mentions `EventPhase`,
`stop_propagation` or `prevent_default` anywhere in its source. The model is
exported to applications and driven by nobody.

**What an application actually gets.**

```rust
pub fn on_command(&mut self, command: InternedSymbol,
                  handler: impl FnMut() + 'static)      // ui_runtime.rs:5388
```

A closure taking no arguments and returning nothing, selected by an FNV-1a hash
of a name string. Concretely, none of the following is reachable by an
application handler: which node was hit; the phase it is in; the pointer's
position, button, device or pressure; the key or the text; whether the event was
trusted; and there is no way to stop propagation or prevent a default action,
because the handler is not given anything to call those on.

**The direct contradiction.** Selecting a handler by hashing the string written
in `on:click=increment` **is** string-based method dispatch. §2.3 does not merely
fail to describe this mechanism — it names it and excludes it.

**How the gap has been managed so far.** By carving out one signature per case
that proved impossible:

```rust
.on_row_click(id, impl FnMut(usize, u32))       // app.rs:161 — which row, and click count
.on_row_long_press(id, impl FnMut(usize))       // app.rs:178
.on_scroll(impl FnMut(f32))                     // app.rs:193 — leftover wheel delta
```

Each exists because the zero-parameter shape made a specific interaction
inexpressible — `on_row_click`'s own documentation says so: "a per-row `on:click`
cannot [carry this], because every row a builder emits shares one command
symbol". That reasoning generalises to every event; only these three cases got a
bespoke signature.

**Cost to application structure.** Since a handler receives nothing, it must
*capture* everything, and capture must happen before the builder chain starts.
The canonical shape is `examples/file_manager.rs:365-372` — eight consecutive
`state.clone()` calls, one per handler. An application can hide the repetition
behind an adapter, which is the workaround this audit's test project uses:

```rust
fn bind<S: 'static>(state: &Rc<RefCell<S>>, method: fn(&mut S)) -> impl FnMut() + 'static {
    let state = Rc::clone(state);
    move || method(&mut state.borrow_mut())
}
// .on_command(CMD_INCREMENT, bind(&state, State::increment))
```

That removes the boilerplate and lets a handler be written as an ordinary method
on the application's state — but it cannot conjure a payload that was never
delivered. Anything event-specific remains out of reach by construction.

**Fix direction.** The types §2.3 requires already exist and are already
exported; what is missing is delivery. Widening the handler to receive the event
(`impl FnMut(&UiEvent)`, with the existing zero-argument form kept as a
convenience) would close the largest part of this without new design work, since
the model it would carry is the one already written and tested.

**Relation to 2-B and 2-C.** All three are facets of one command model that
addresses handlers by hashed name: 2-B is the *routing* half (only link 5 of 5 is
reachable), 2-C is the *validation* half (no name is ever checked), and this is
the *payload* half (nothing is delivered). Fixing any one alone leaves the model
lopsided.

---

## Tier-2 — spec-required gaps

### 2-A. `App::screen` materialises a `.okui` screen once (§2.9)

**Closed for values in `087a3cf`.** `.screen()` keeps the compiled IR rather than the
built `View` and re-lowers it per frame against a `ScreenData` table — 6.0–6.4µs
per frame unoptimized, no re-parse, no re-compile. Structure is still materialised
once (`<If>`/`<For>` are fixed), which is now stated on the method. The
"mutually exclusive" claim below is void: `build.rs` is open since 1-A, and the
run-time route now carries state.

```rust
Some(Root::Screen(src)) => {
    let view = compile_screen(&src)?;
    Box::new(move || view.clone())      // app/app.rs:231
}
```

The builder returns the same pre-built `View` every frame, so no state change can
reach the screen. §2.9 defines `.okui` as a frontend "compiled into the same UI
IR as the Rust DSL" — a peer of the Rust DSL, not a static subset — and requires
**compile-time** contract validation, whereas `.screen()` compiles inside `run()`.

The limitation itself *is* explained, and well, in
`examples/file_manager.rs:10-13`. The problem is placement: that is a comment on
one of 53 examples, while `09-authoring-formats.md:39` — the page a reader
consults when choosing between the two frontends — says only "compiles the markup
at `run`" and presents both as equivalent options ("Three ways to write a view",
`:3`). `app.rs:228` calls it "static v1 materialization" internally.

**Consequence worth stating in the docs.** A standalone `.okui` file and
reactivity are mutually exclusive today: `.screen()` is static and the `build.rs`
route is blocked by 1-A. Markup over changing data means the inline `okui!` macro,
and only that.

### 2-B. `App` exposes one of the five command-routing links (§2.6)

**Closed in `087a3cf`.** `App::on_command_at` and `App::add_event_listener` name their
target with a §7.5 `SelectorTarget` — a `test-id`/`automation-id` authored on the
markup — resolved after each commit, so a NodeId is genuinely obtainable and a
registration follows the *name* across rebuilds. §2.6's focus-scope link has no
first-class method and goes through `AppControl::with_runtime`.

§2.6 specifies routing verbatim: focused node → focus scope → ancestors → window
→ application fallback. `UiRuntime` implements it, including per-node
registration:

```rust
pub fn on_command_at(&mut self, node: NodeId, command: InternedSymbol,
                     handler: impl FnMut() + 'static)     // ui_runtime.rs:5548
```

whose own documentation states the point: "This is what makes routing
*positional*: the same command symbol can run different logic depending on where
focus sits."

`App::on_command` (`app.rs:211`) registers `CommandTarget::App` — link 5, the
global fallback — and `App` never yields the runtime, so no `NodeId` is
obtainable and links 1–4 are unreachable from a standalone application.

Observable costs: one flat command namespace keyed by a string hash; the same
symbol cannot mean different things in different places; and handlers cannot sit
beside the markup they serve.

**The runtime is conformant here.** The gap is in the entry point — unlike 1-C,
where the runtime bypasses the spec's model itself. See 1-C for how routing,
validation and payload compose into one problem.

### 2-C. Command names are never checked (§2.13)

**Half closed in `9f4b235`, and the other half is a residue by construction.**
`CommandSurface` / `Compiler::claiming_command` check the file front end, with
`obscur-kit`'s own `build.rs` as the production caller. The `okui!` macro cannot be
checked: it expands while the application crate is still compiling, so the
registrations do not exist yet, and a false "unknown command" would be worse than
the silent no-op. Two wires remain: `App` does not declare its commands to the
run-time `.screen()` route, and the runtime could warn once when a command reaches
the end of §2.6's chain with no handler anywhere — the only place the true set is
known.

§2.13: "Unknown attributes, events and **string method names** are compile
errors." The handler reference in `on:click=increment` is exactly such a string
reference: it is hashed to a symbol (`ui_ir_view_bridge.rs:123`) and matched
against whatever `on_command` registered. A mismatch is a silent no-op — no
compiler error, no runtime diagnostic, a button that simply does nothing.

The hashing itself is sound: `name_binding::fallback_symbol` (`:414`) and
`InternedSymbol::named` (`interned_symbol.rs:35`) are the same FNV-1a with the
same high-bit mask, pinned equal by a drift test. The gap is that nothing
verifies a name is *claimed* by anybody.

### 2-D. No path to explicit stylesheet invalidation through `App` (§4.12)

**Closed in `087a3cf`.** `AppControl::set_stylesheets` reinstalls the authored set
through the retained pipeline, and the set is handed over as identified, versioned
sources instead of the anonymous ones §4.12's cache cannot key. Granularity is
coarse: a replacement stamps a new revision on every sheet in the set.

§4.12 requires that "CSS parsing is cached per unique source/version and
invalidated **explicitly**". The mechanism exists: `CssSource` carries a
`SourceVersion` (`css_source.rs:48`) and `UiRuntime::set_stylesheets`
(`ui_runtime.rs:2772`) replaces sheets at any time.

`App` collects authored sheets once before the loop starts (`app.rs:109`
accumulates; nothing re-supplies) and never exposes the runtime, so an
application cannot invalidate anything. Every style edit costs a rebuild.

Same shape as 2-B, same one-line remedy: let `App` hand out the `UiRuntime`.

### 2-E. A contract cannot express "no children" (§2.13, §2.12)

**Closed in `9f4b235`.** `ChildPolicy` distinguishes the three states the empty list
conflated, and text content counts — a `"text"` child carries no tag, which is how
this document's own `<Button>OK</Button>` escaped the check. The claim is about the
component spelling: `445b6bd` made lowercase `<button>` a base element whose
authored children are what renders.

§2.13 requires a contract to carry `allowed children`; §2.12 adds slot
cardinality. `validation/builtin_contracts.rs:99` states the shortfall plainly:
an empty `allowed_children` means *any* child, and the type has no way to say
*none*, so `<Button>OK</Button>` — meaningless, since every widget except
`Dialog` synthesises its own children — still compiles.

---

## Tier-3 — documentation defects

All four are in `docs/api/09-authoring-formats.md`, which is otherwise the page an
author relies on most.

**All four closed in `e238817`**, together with the drift the page had accumulated
beyond them: it now opens with four *routes* rather than three ways, because a
`.okui` file has two and they are not equivalent. One correction to 3-A's own
wording: `<Tabs/>` with a child is a compile error since `9f4b235`; bare `<Tabs/>`
is still silent.

### 3-A. The reason given for collection widgets being Rust-only is wrong (`:69`)

The page says `List`, `Table`, `Tabs`, `Tree`, `Menu` and `Docking` "have no
markup contract, so they are reachable only from Rust". They do have contracts:
`widget_contracts()` registers all twelve and `covers_every_shipped_widget`
asserts exactly twelve (`builtin_contracts.rs:105`, `:170`), and the tags resolve
to real `ComponentId`s (`name_binding.rs:261-267`).

The real reason is in the source (`builtin_contracts.rs:92`): their content is
typed Rust values (`MenuItem`, `TreeNode`, `Column`, `DockPanel`) with no string
spelling, so their surfaces were deliberately left **open**. The practical effect
is worse than the page implies — `<Tabs/>` compiles and mounts an empty widget
silently, where the reader expects a compile error.

### 3-B. `<Slot>` is listed as control flow (`:64`)

The outlet is lowercase only: `pub const OUTLET_TAG: &str = "slot"`
(`name_binding.rs:161`). The source spells out the consequence at `:155-160` — a
`<Slot/>` in component position "stays what any unrecognised component tag is —
`user_component_id("Slot")`, **mounting nothing**".

An author copying the documented spelling gets a node that mounts nothing and no
diagnostic at all. The same mechanism means any typo in a PascalCase tag
(`<Buton/>`) passes silently: §2.13 closes typos in *attributes* of a known
component, but nothing closes typos in *component names*.

### 3-C. Four of the nine reserved attributes are listed (`:54`)

The page names `class`, `id`, `type`, `src`. `attr_symbol`
(`name_binding.rs:371`) also reserves `disabled`, `checked`, `test-id`,
`automation-id` and `slot`. Unlisted names are not rejected — they are hashed
into ordinary props — so the omission misleads in both directions.

### 3-D. Three spellings of an `on:` value across the documentation

| Spelling | Source |
|---|---|
| `on:click={on_ok}` | `09-authoring-formats.md:25` |
| `on:click="save"` | `interned_symbol.rs:29` |
| `on:click=submit` | frontend tests, e.g. `okui-syntax/src/ast/attribute.rs:132` |

The distinction that is never drawn: inside `okui!`, `{ident}` means **two
different things by position** — in body position it is a Rust binding from the
enclosing scope, in an `on:` attribute it is a *name*, not an expression.
`examples/file_manager.rs` demonstrates both in one function: `{path}` is a
local, while `goParent` appears only in the markup and in
`command("goParent")` — there is no Rust binding of that name anywhere.

---

## Withdrawn after checking the spec

Recorded because a reader should be able to see what was rejected.

- **`@import` is unsupported.** True — the parser knows only `@media`,
  `@container`, `@keyframes`, `@-webkit-keyframes` — but §4.13 lists `@import`
  under "Excluded or restricted". Intended, not a defect. CSS is split across
  files by repeated `.stylesheet()` calls (which accumulate, `app.rs:105`) or by
  a preprocessor via `build.rs`.
- **"CSS must be inlined in Rust."** False. `.stylesheet()` takes `&str`;
  `include_str!` and a `build.rs` → `OUT_DIR` → `include_str!` preprocessor
  pipeline both work, and 1-A does not reach CSS because CSS stays text.
- **An early observation that the sample app exited immediately** did not
  reproduce and was an artefact of the harness, not the kit.

---

## Reproducing this

The audit application is a separate crate outside the workspace, depending on
`obscur-kit` with `window-winit` + `render-vello`.

The measurement harness was lifted into the repository — it is
`checks/facade-consumer/src/measure.rs`, in the out-of-workspace consumer crate
`191d79e` added, so the vantage point this audit was written from is now permanent
rather than a thing someone has to rebuild. It was worth lifting regardless of
these findings — it answers layout questions with numbers instead of argument, needs no
display or GPU, and is about forty lines:

```rust
let source = CssSource::anonymous(css);
let parsed = StylesheetParser::new(&source).parse();
let authored = StylesheetCompiler::new().compile(&parsed).matchable;

let mut runtime = UiRuntime::new(RuntimeId::from_raw(1), Size::new(w, h));
runtime.render_now(&build_view(), &[ua_stylesheet(), authored]);

for node in runtime.inspect_snapshot().nodes {
    println!("{node}");
}
```

`examples/showcase_inspect.rs` is the in-repo ancestor of this; the difference is
only that the version above takes the stylesheet from a file, which is what made
the 1-B experiment table cheap to produce.
