# Decomposition: the god object, then the whole workspace

Two campaigns, one law. The **first** asked whether one file's *state* was one
entity's state, and took `UiRuntime` apart along the seams that answer exposed;
it is everything up to and including *The second pass*. The **second** asked
§3.16's other half — *how many main entities does this file hold?* — of the whole
workspace, under the owner's operational bar of **at most 1,000 lines of non-test
code per file**, and worked down the twelve largest files that were over it: *The
third pass* onward, plus *The passes that closed without a chapter*, ending at
*Closing the books*, where the bar stops being a convention and becomes a check
`xtask` runs.

Neither campaign invented its rule. Spec §3.1 admits one main public or
domain-relevant entity per file, its `impl`, private local functions **with no
independent semantics**, and that entity's unit tests; spec §3.16 requires that
"один файл не экспортирует несколько главных сущностей без явного exception
marker". A file over the bar is the *symptom*. Several entities in one file is
the disease, and the number is only ever the thing that made someone look.

**Status:** the map is exhausted. **Twelve entities extracted, seven clusters
examined and refused** — every row of the original cluster map has now been
resolved one way or the other, and a refusal is a result: the reasoning is
written down below so the next reader does not re-litigate it.

Measured on the tree as it stands, not copied from a stage report:
`ui_runtime.rs` is **17,371 lines with 47 fields**, its `impl UiRuntime` is
**6,433 lines over 221 methods** (131 of them `pub`), and **9,969 of the file's
lines — 57% — are its `#[cfg(test)] mod tests`**. Suite 6,099 passed / 0 failed,
`clippy` 391 workspace warnings, `fmt` clean, `xtask check` and `xtask consumer`
both pass.

**Those figures are that campaign's close, not the tree's today.** Seven passes
have run since — the harness arc, the facade pass, the frame pass, the input
pass and the sweep — and `ui_runtime.rs` is now **1,834 lines, 176 of them
non-test**, with **no `impl UiRuntime` at all**: the struct, its field docs, 31
child-module declarations and `Debug`. Suite 6,229 / 0, `clippy` 362. Each pass
states its own measurement in its own chapter; read the last one for the current
number.

What is left in the struct is the frame pipeline itself plus the eleven fields
rule 1 refused; both are enumerated in **Remaining**, below.

## Why

`crates/application/obscur-runtime/src/runtime/ui_runtime.rs` was **19,836 lines
with 103 fields and 254 methods in one `impl`**. Spec §3.1 admits "one main
public or domain-relevant entity, its `impl`, private local functions with no
independent semantics, and that entity's unit tests" per file — and formally
`UiRuntime` is one entity, which is exactly how the file got here. The 103 fields
are the evidence that it is not: they are a dozen entities' state flattened into
one struct, each reachable from all 254 methods.

The spec had already named the shape. `crate-arch §3.1` forbids dumping-ground
names and asks for semantic ones (`frame_scheduler.rs`, `resource_hub.rs`,
`focus_scope.rs`); the prescribed `obscur-runtime` layout in `crate-arch §3` puts
coordinators in `orchestration/` and runtime-owned entities in `runtime/`. Every
extraction below lands in that layout.

## A caveat about the "byte-identical render" claims

Several chapters below report that `showcase_png` rendered **byte-identical**, and
quote a digest. **Those digests are not comparable across chapters** — eight
distinct ones appear in this document (`d1e04c9b…`, `bb153594…`, `0544d85c…`,
`19166649…`, `220cf2ca…`, `47224af9…`, `9f88b478…`, `9f936a94…`). Each was
measured in its own session against its own immediately-prior state, and the
showcase's output legitimately depends on which system font `fontconfig` resolves
and on which of the example's two outputs (`showcase_cpu.png` at scale 1, or
`showcase_hidpi.png` at scale 2) that run produced. A chapter's digest therefore
is evidence for only "this step changed nothing", never "the render has never
changed"; reading the list as a chain would be reading it wrong.

The durable evidence that the render path survived these campaigns is not a hash
but the suite: 137 test files in `crates/facade/obscur-kit/tests/`, among them
`visual_css.rs`, `grid_css.rs`, `layout_css.rs`, `spec_ch5_paint_order.rs`,
`relative_units_paint.rs` and `image_css.rs`, which assert paint output against
the spec rather than against a previous artifact. They are in the green suite each
chapter reports.

## Method

Each method in the `impl` was mapped to the state it touches, and the clusters
fell out of the field usage rather than out of a naming intuition. The rules the
extractions follow:

1. **A cluster becomes an entity only where its fields share an invariant.** The
   test is whether some operation must touch two of them together — releasing an
   evicted image's hub handle, dropping a closed scope's handlers. A cluster with
   no such pairing is a bag of fields and stays where it is.
2. **The entity takes what it needs as arguments.** No extracted type holds the
   tree, the viewport or the diagnostics sink; the runtime hands them in. That is
   what turns an implicit `self` dependency into a visible one.
3. **The public API does not move.** `UiRuntime`'s methods keep their signatures
   and their doc comments — a user reads the docs on the surface they call — and
   only the bodies delegate.
4. **A method with no production caller is not created.** Where a test needed one
   (`ResourceBridge::state_of`, `HandlerRegistry::remove_command`) it is
   `#[cfg(test)]` and says so, rather than widening the surface to serve a test.

## Done

| Entity | Fields taken | Lines | Invariant it now owns |
|---|---|---|---|
| `runtime/resource_bridge.rs` — `ResourceBridge` | 14 | 1,129 | An evicted image must release its hub handle, or the URL reads as "already attempted" and never loads again |
| `runtime/handler_registry.rs` — `HandlerRegistry` | 10 | 562 | Closing a focus scope drops its handlers; clearing a node drops both its handlers and its listeners |
| `runtime/pointer_sessions.rs` — `PointerSessions` | 8 | 758 | A lift or a cancel ends a pointer's press, pan, hold, capture and drag-selection **together** — a teardown that was written out by hand in the `Up` arm and again in the `Cancel` arm |
| `runtime/input_queue.rs` — `InputQueue` | 4 | 198 | The latch that redirects a handler's own `request_frame`, and the plan it redirects into, are only correct as a pair — and a plan merged twice applies the same invalidation twice |
| `runtime/surface_geometry.rs` — `SurfaceGeometry` | 4 | 297 | The render transform and the pointer mapping must stay exact inverses; the viewport is derived from the *clamped* device scale, so the two cannot disagree |
| `runtime/motion_driver.rs` — `MotionDriver` | 9 | 870 | Installing a stylesheet clears all three change-detection baselines together — a sheet re-observing against a stale one either misses the first change or replays an animation that already ran |
| `runtime/frame_telemetry.rs` — `FrameTelemetry` | 2 | 370 | A phase's cost count is *both* the counter's value and the number the budget is checked against; taken twice at a call site, the violation can describe different work from the counter beside it |
| `runtime/background_work.rs` — `BackgroundWork` | 2 | 178 | A serviced unit leaves the pending queue exactly as it enters the serviced total, and the scheduler's pending/idle signal is the `0 → 1`/`1 → 0` edge of that same count |
| `runtime/scroll_offsets.rs` — `ScrollOffsets` | 3 | 455 | The scroll-folded box projection is a *cache* of the offset map, so a write that moves an offset must rebuild it in the same call — or the public geometry accessor reports a node at the position it was painted at before the scroll |
| `runtime/virtual_lists.rs` — `VirtualLists` | 7 | 738 | A realized window, the row extent it was realized at and the container it landed in are one fact recorded by one call — a window without its extent leaves every row of that list unclickable, silently — and a replaced model's scroll reset must land *before* the window that reads that scroll |
| `runtime/cascade_sheets.rs` — `CascadeSheets` | 3 | 326 | What a frame remembers is the **author** half of the list it cascaded, never the list itself — a retained copy of the UA sheet outlives the sheet that replaces it and then beats it on source order, so the one sheet in the cascade that a host can reinstall becomes the one it cannot |
| `runtime/accessibility_publisher.rs` — `AccessibilityPublisher` | 2 | 357 | The focus a push *reports moving from* and the focus it *records having published* are the same field, written by the same call — a push that fails to record re-walks the whole retained tree on every frame that follows it, forever, and tells the platform a transition that did not happen |

Twelve files, **6,238 lines**, **68 fields taken**. The field arithmetic closes
exactly, which is worth writing down because it is the campaign's only
unambiguous measure: 103 − 68 taken + 12 handles the runtime keeps in their
place (`resources`, `routing`, `pointers`, `input`, `surface`, `motion`,
`telemetry`, `background`, `scroll`, `virtual_lists`, `sheets`, `accessibility`)
= **47**.

`ui_runtime.rs`: **19,836 → 17,371** lines, **103 → 47** fields, **254 → 221**
methods. Suite 6,020 → 6,099 — and the 79 is not an estimate either: the twelve
new files carry exactly 79 `#[test]` functions between them, so every test the
suite gained is a test that drives an extracted entity, and **no test was
weakened, rewritten or deleted** to get there. `clippy` 392 → **391**; the one
warning that went away is a `let_and_return` the virtual-list extraction removed
rather than carried, and the only `let_and_return` left in the workspace is in an
unrelated test file (`crates/facade/obscur-kit/tests/duplicate_key_diagnostic.rs`).

`showcase_png` renders byte-identical at every step that touches the scene path.
Where it proves nothing, the ledger says so rather than citing it anyway:
`FrameTelemetry`, `BackgroundWork` and `AccessibilityPublisher` sit on the frame
path but produce no display list, and `ScrollOffsets` and `VirtualLists` move
code the showcase never runs (it declares no `overflow` and no `<VirtualFor>`).
Each of those was proved by the integration tests that *do* exercise the path,
named in their sections below.

Saying so matters most for `ScrollOffsets`, because the gap is the widest there:
the showcase declares no `overflow` anywhere, so it establishes no scroll
container, and the code that step moved never runs in it. Its scene-path proof is
the three examples that *do* scroll — `scroll_proof` (clip + offset + scrollbar +
scrolled hit-test), `textarea_vscroll_proof` and `caret_scroll_proof` — each of
which asserts its own geometry and still passes. `VirtualLists` is proved the
same way, by the sixteen virtual-list integration tests; `CascadeSheets`, by
contrast, is a case where `showcase_png` is a *real* proof rather than a
formality — the showcase installs a UA sheet and author sheets and runs exactly
the cascade that step rewrote, and `showcase_hidpi.png` came out byte-identical
(`d1e04c9b…`).

The line count moves more slowly than the field count, and that is the honest
shape of the work: a public method keeps its doc comment on `UiRuntime`, where a
caller reads it, and only its body delegates. What leaves is state and the logic
that guards it — which is what the 103 → 47 says.

There is a second reason, and it is measurable: **9,969 of the file's 17,371
lines are its `#[cfg(test)] mod tests`**. The production body — struct, `impl`,
free helpers and the `Debug` impl — is 7,402 lines, and the `impl UiRuntime`
block alone is 6,433. So the file is majority test, and no extraction of this
kind can move that half: an extracted entity takes its *own* unit tests with it,
while the runtime's integration-shaped tests exercise `UiRuntime`'s public API,
which rule 3 keeps exactly where it is. Anyone reading the 17,371 as "the god
object barely shrank" should read the 6,433 instead.

### What these extractions found

One defect turned up **ten times**, in eight of the twelve extractions and in two
of the refusals, and it is worth naming as a class rather than as ten
coincidences. The shape is always the same: **two contiguous `///` blocks with
the blank line missing between the items they belong to**, so `rustdoc` reads
them as one comment on the *second* item. The first item is then silently
undocumented and the second one's documentation opens by describing something
else — sometimes a hundred-odd lines away. Nothing in the toolchain flags it:
it is valid Rust, `rustdoc` renders it happily, `clippy` has no lint for it, and
`missing_docs` is satisfied because the doc *is* attached to something. In a
17,000-line file it is invisible by construction. All ten are fixed, each text
restored verbatim to the item it describes.

| # | Found by | The item left bare | Where its doc had gone |
|---|---|---|---|
| 1 | `ResourceBridge` | `size_replaced_elements` | the item below it |
| 2 | `ResourceBridge` | `decode_one_image` | the item below it |
| 3 | `ScrollOffsets` (method half) | `scroll_top` | `dump_virtual_geometry` — dragging a `#[must_use]` onto a unit-returning function with it |
| 4 | editing (refused) | `reseed_editing_from_retained` | `declared_control_value`, as a combined 37-line block — including the whole "reseed discriminator" section, the only written account of the controlled-input gate |
| 5 | editing (refused) | `scroll_paint` | `editing_view` |
| 6 | `FrameTelemetry` | `record_phase` | the head of `LayoutInputs`' doc, 141 lines below — so a struct's documentation opened by describing a free function |
| 7 | `FrameTelemetry` | the `telemetry` **field** | nowhere: bare between two documented neighbours |
| 8 | `ScrollOffsets` | the `scroll_top` **field** | nowhere: same missing blank line seen from the other side, the *item* bare rather than the doc misplaced |
| 9 | `VirtualLists` | `run_frame` | the head of `virtual_window_len`'s doc — so the window-length helper opened by describing the frame pipeline, and the frame pipeline's only written account of `FrameReason` → `FramePlan` classification sat on a two-line helper |
| 10 | `VirtualLists` | — | the far side of #1: nineteen lines about replaced elements still glued above `size_virtual_spacers`' doc after the text had been copied to its real home. Removed as a duplicate |

The last two extractions, `CascadeSheets` and `AccessibilityPublisher`, found
**none** — recorded so that the streak breaking is visible rather than assumed.

Three pieces of residue, all removed in the new home rather than carried over:
`dump_virtual_geometry` was still called as `let () = self.dump_virtual_geometry();`,
the binding forced by the `#[must_use]` that defect #3 had dragged onto it long
after the attribute itself was gone; `track_virtual_containers` bound its loop's
node id and then discarded it with `let _ = id;`, a dead binding laundered
through a no-op instead of destructured away; and the realize block took
`virtual_windows` out with `std::mem::take`, put it straight back unchanged and
only then extended it — a round trip that did nothing. The same block ended in a
`let`-then-return that `clippy` was warning on, which is the entire 392 → 391
difference in the workspace warning count.

One duplicate: `active_animation_count` and `active_animations` are the same
reading arrived at by two routes (§7.6's DevTools counter and §4.17's
keep-ticking signal). Both are public, so both stay — but each now says so, and
the driver has one method under them.

**Nine behaviours were found and deliberately not changed.** Each is a real
defect or a real ambiguity, and each is documented where it lives and covered by
a test that *names* the behaviour rather than endorsing it, so that the change
that fixes it has somewhere to start. A refactor that silently alters behaviour
is a refactor that cannot be reviewed; that is the whole reason for the list.

| Where | What | Section |
|---|---|---|
| `reseed_editing_from_retained` | re-derives only the *horizontal* caret-follow scroll, where `after_edit` re-derives both axes — an externally shortened `<textarea>` keeps a `scroll_top` computed for the old content | editing |
| `BudgetViolation` | reaches the telemetry sink only; `to_diagnostic` exists and `frame_budget.rs` says it "flows through the normal diagnostics channel", but nothing routes it there | diagnostics |
| `PerformanceSpan` | carries no duration, so §7.9's timing tier emits markers, not measurements — by construction, since the runtime is clock-free | diagnostics |
| `scroll_by` | never checks `dy`, and `(current + NaN).clamp(0, max)` is `NaN` in Rust rather than a panic, so a `NaN` wheel delta reaches `ScrollPaint` and the folded projection. Pinned by `a_non_finite_shift_delta_poisons_the_offset_and_should_not`, which asserts the **wrong** behaviour as a tripwire and says so — invert it when the guard lands | scroll |
| `clamped_scroll_top_of` | claimed a clamp it could not perform; the claim is now honest and the method is named `stored` so the weaker guarantee is visible at the call site *(this one was corrected)* | scroll |
| `set_stylesheets(vec![])` | cannot turn styling off: "I own no styling" and "drop the author styling" are the same value, and the retention reads it as the first | sheets |
| `set_default_stylesheet` | its "any other origin still works" holds at the top level only; `scoped_internal_declarations` tests `SystemDefault` exactly, so a default sheet at another origin is scoped *out* of every component-internal node | sheets |
| `push_accessibility` | the host route and the installed-bridge route share one `published_focus` record, which makes them alternatives rather than parallel outputs | a11y |
| `DragCoordinator::release` | is not `#[must_use]`, and `Option` is not `#[must_use]` as a type (checked, not assumed), so dropping its return loses a validated `TransactionCommit` with no warning anywhere | drag |

Plus `set_accessibility_bridge` not resetting the published-focus record, which
is the tenth and the one genuinely open *question* rather than a defect: which
reading of the port's contract is right (`focused` or `changed`) is a decision.

### What the pointer extraction changed beyond moving code

The recognizers are now **pure**: they answer what the contact is doing and
return the effect rather than performing it. `update_pan` returns a `PanStep` the
runtime scrolls by; `advance_flings` returns distances the runtime applies;
`advance_holds` returns the holds that fired for the runtime to dispatch. That is
what makes them testable without a runtime at all — the seven new unit tests
drive gestures directly, including the two the old shape could not reach: that a
cancelled pointer throws nothing however fast it was moving, and that crossing
the slop discards the press, drops the drag-selection and takes the capture in
one step.

## Remaining

**Nothing on the original map is outstanding.** Every cluster it named has been
extracted, split, or examined and refused, and the refusals have their reasoning
in the sections below so that the next reader inherits the argument rather than
the question:

| Cluster the map named | Fields | Outcome |
|---|---|---|
| ~~resources~~ | 14 | extracted — `ResourceBridge` |
| ~~handlers~~ | 10 | extracted — `HandlerRegistry` |
| ~~input~~ | 16 | **split three ways**: `PointerSessions` (8), `InputQueue` (4), and 4 fields refused |
| ~~viewport / surface~~ | 6 | **split**: `SurfaceGeometry` (4), `scroll_top` reassigned to its own cluster, `environment` refused |
| ~~motion~~ | 9 | extracted — `MotionDriver` |
| ~~diagnostics / telemetry~~ | 6 | **split four ways**: `FrameTelemetry` (2), `BackgroundWork` (2), `diagnostics` and `inspect_mode` refused |
| ~~per-container scroll~~ | 1 (+2) | extracted — `ScrollOffsets`, which took the folded-box cache with it |
| ~~virtual lists~~ | 7 | extracted — `VirtualLists`, which took the realization block out of `run_frame` with it |
| ~~stylesheets~~ | 4 | **split**: `CascadeSheets` (3), `locale` refused |
| ~~drag / a11y~~ | 4 | **split**: `AccessibilityPublisher` (2), the drag half refused |
| ~~editing~~ | 1 | **examined and refused** — the largest cluster by line count and the clearest refusal |

### What the 47 fields are now

Measured, and it accounts for all of them:

- **12 are handles to the extracted entities.** `resources`, `routing`,
  `pointers`, `input`, `surface`, `motion`, `telemetry`, `background`, `scroll`,
  `virtual_lists`, `sheets`, `accessibility`. Each replaced between 2 and 14
  fields, which is where 103 − 68 + 12 = 47 comes from.
- **24 are the frame pipeline**, which is what `UiRuntime` is *for* and what it
  should be left holding: `id`, `tree`, `scene`, `reconciler`, `enactor`,
  `matcher`, `style`, `layout`, `measure_cache`, `arrange_cache`,
  `scene_composer`, `scheduler`, `boxes`, `inputs`, `computed`,
  `generated_content`, `overlaid`, `last_style`, `spatial`,
  `last_layout_inputs`, `next_frame`, `staged_plan`, `root_view`, `components`.
  These are not an unexamined remainder — they are the reconcile → style →
  layout → compose sequence and the caches that let a frame skip phases of it,
  and every one of them is read by the pipeline methods that stayed.
- **11 were examined and refused.** `hover`, `focused`, `pseudo`, `modifiers`
  (input's leftovers), `environment` (the viewport's), `editing`, `diagnostics`,
  `inspect_mode`, `locale`, `drag`, `drag_outcome`. Each has its reasoning in a
  section below.

### Where the next reduction is, and what it is not

It is **not** another extraction of this kind. Three things are worth saying to
whoever picks this up:

1. **The refusals are load-bearing and should not be re-opened without a new
   argument.** Seven clusters failed rule 1, and three of them failed *inside* a
   cluster that partly passed — rule 1 is a per-pair test, not a per-cluster one.
   The two strongest counter-proposals (`TextEditor`, a drag wrapper) were each
   tested against the same three criteria — does it move state, can it own its
   candidate invariant, does anything become newly testable — and each failed all
   three. Re-proposing one needs to answer those three, not restate the shape.
2. **The file's remaining bulk is tests, not logic.** 9,969 of 17,371 lines; the
   `impl` is 6,433. Splitting the test module by concern would move more lines
   than any remaining extraction could, and it is a different job with a
   different justification — spec §3.1 admits "that entity's unit tests" in the
   file, so the cap in `file_size.rs` is the only thing arguing about it.
3. **The named follow-ups already exist**, and each is its own change with its
   own test: the ten reported-not-changed behaviours in the table above, and the
   ~150 lines of pure session functions that should grow onto `TextEditSession`
   (the constructive remainder of the editing refusal). The next campaign target
   named by the `file_size.rs` ledger was `orchestration/scene_coordinator.rs`,
   not this file — see **the third pass**, below, which has since done it.

## The clusters one by one, including the refusals

Each section below is the argument for one row of the map — why a cluster became
an entity, or why it did not. The refusals are here on purpose and at the same
weight as the extractions: **seven clusters failed rule 1**, three of them inside
a cluster that partly passed, and a refusal without its reasoning written down is
just an invitation to try the same extraction again next quarter.

Eleven fields over eight rows, which is seven cluster failures — the first two
rows are both the input cluster's leftovers.

| Refused | Fields | The short reason |
|---|---|---|
| `hover`, `focused`, `pseudo` | 3 | read across the whole runtime and sharing no invariant with each other; extracting them buys a wide accessor surface and no guarantee |
| `modifiers` | 1 | one `Copy` field folded from key transitions and read by three methods — nothing to guard |
| `environment` | 1 | one field read by four methods with nothing to pair it with |
| editing | 1 | its internal invariants are **already** owned by `TextEditSession`; what is left on `UiRuntime` is orchestration against six things rule 2 forbids an entity from holding |
| `diagnostics` | 1 | one sink, and *no operation touches it together with a second field* — a budget violation and a resource failure never meet |
| `inspect_mode` | 1 | one `bool` its apparent neighbours never read; they read the tree, the boxes and the computed styles |
| `locale` | 1 | its only companion is the scene coordinator (rule 2), and the pairing it does have is already owned by `LayoutInputs` |
| `drag`, `drag_outcome` | 2 | `DragCoordinator` is already an entity; the remaining field is a one-slot host mailbox no drag operation reads |

### What the input cluster left behind, and why

The original map counted "input" as one 16-field cluster. It is not. Splitting it
by the invariant test gave two real entities — `PointerSessions` and `InputQueue`
— and four fields that failed the test and stayed:

- **`hover`, `focused`, `pseudo`** are read across the whole runtime (the
  semantic tree, both restyle paths, key handling, command dispatch,
  accessibility). They share no invariant with each other that some operation
  must maintain; extracting them would buy a wide accessor surface and no
  guarantee. That is rule 1 refusing a cluster, which is the rule doing its job.
- **`modifiers`** is one `Copy` field folded from key transitions and read by
  three methods. There is nothing to guard.

The same happened to the "viewport / surface" cluster, which was three things:
`SurfaceGeometry` took the four that form one coordinate system, `scroll_top` is
a separate concern (per-container scroll, whose clamping needs the live layout),
and **`environment`** is one field read by four methods with nothing to pair it
with. Two of six clusters on the original map turned out not to be clusters —
which is the map being a hypothesis and the invariant test being the answer.

### Why editing — the largest cluster — stays where it is

The editing cluster was measured method by method: **32 methods, 1,203 lines**
(the original estimate of 19/690 counted bodies only). The measurement is also
the refusal. Excluding calls between the 32, here is the state they touch:

| State | Methods touching it |
|---|---|
| `editing` | 19 |
| `tree` | 8 |
| `computed` | 5 |
| `boxes`, `scene_composer` | 4 each |
| `routing` (via `dispatch_*`), `request_frame` | 3, 2 |
| `pointers`, `scroll_top`, `focused`, `modifiers` | 2, 1, 1, 1 |

Exactly **one** field is the cluster's own. Rule 1's test — some operation must
touch *two* of its fields together — has no candidate pair, and the invariants
that *do* exist inside a session (buffer ↔ composition ↔ `committed` ↔ `mirror` ↔
`scroll_left`) are **already owned** by `TextEditSession`, which is already an
entity in its own file. What is left on `UiRuntime` is not state with a guard
around it; it is orchestration *between* the session and the tree, the styles,
the retained boxes, the scene coordinator, the handler registry and the frame
scheduler — every one of which rule 2 forbids the entity from holding.

Three tests were applied to the strongest counter-proposal, a `TextEditor` owning
the session plus the orchestration, and it fails all three:

1. **It moves no state.** `Option<TextEditSession>` becomes `TextEditor`:
   60 fields → 60. This ledger's own measure of progress does not move, because
   what would move is lines, not "state and the logic that guards it".
2. **It cannot own its one candidate invariant.** That candidate is real —
   *every route that changes the buffer must re-derive the caret-follow scroll on
   both axes and schedule exactly one frame*. But its second axis is
   `scroll_top`, which the wheel and the scrollbar write too and which this table
   lists as a **separate** cluster. Either the editor reaches into another
   cluster's map on most calls, or — the `SurfaceGeometry` precedent, which
   *reports* a reband rather than dropping a cache — it returns the `scroll_top`
   it wants and the runtime applies it, leaving the runtime as the thing that
   must still remember to do both. Enforcement evaporates either way.
3. **Nothing becomes testable that is not already.** The genuinely pure parts
   (grapheme-cluster range, word/line bracketing, the IME follow-up decision)
   need only a session, so they are already reachable through `TextEditSession`.
   The rest — caret offset at a point, caret scroll, the compose-only
   classification — need a laid-out tree, computed styles, retained boxes and a
   scene coordinator, so their tests must build a runtime whatever owns them.
   Contrast `PointerSessions`, whose recognizers became *pure* and whose seven new
   tests drive gestures with no runtime at all. There is no such prize here.

The constructive remainder is a smaller, different job: roughly 150 of the 1,203
lines are pure functions of the session and should **grow onto
`TextEditSession`**, which is what "already has `text_edit_session.rs` to grow
into" was pointing at. That is a tidy pass, not an extraction — it does not
reduce the field count either — and it is deliberately not done here.

Three clusters on the original map have now failed the invariant test (input's
leftovers, viewport's `environment`, and editing). That is the map being a
hypothesis and rule 1 being the answer.

### Why the diagnostics cluster is two entities and two leftovers

The map counted "diagnostics / telemetry" as one six-field cluster. Applying
rule 1 per pair splits it four ways:

- **`telemetry` + `budget` → `FrameTelemetry`.** One operation must touch both:
  a phase's deterministic cost count is the counter's value *and* the number
  checked against the bound, and the violation lands in the same sink under the
  same subsystem. That operation was written out at four call sites, each passing
  the count twice and naming the bound by hand
  (`record_phase(&mut *self.telemetry, "layout", …, count, self.budget.check_layout(count), …)`).
  `record_phase(FramePhase::Layout, count, frame)` now takes the count once and
  derives the span, the label, the subsystem and the bound from the phase.
- **`background_work` + `serviced_background` → `BackgroundWork`.** `service`
  is the only writer of either and must write both; and the scheduler's
  pending/idle signal is the `0 → 1`/`1 → 0` **edge** of the same count, which
  used to be an `== 1` guard in one method and an `== 0` guard in another. The
  type reports the edge and the runtime signals the scheduler — the
  `SurfaceGeometry` precedent, because rule 2 forbids it holding the scheduler.
- **`diagnostics` stays.** It is one sink with no second field any operation must
  touch it with. Folding it into `FrameTelemetry` would be inventing an
  invariant: a `BudgetViolation` goes to the telemetry sink, a component or
  resource failure goes to the diagnostic sink, and **no operation touches
  both**. It would also fight rule 2 — `ResourceBridge` already takes the sink as
  an argument (`ResourceReporting`), so the resource path would have to reach
  through a telemetry type to report a failed load. Wrapping it alone moves no
  state (`Box<dyn DiagnosticSink>` → a newtype: 58 fields → 58), which is the
  measure the `TextEditor` proposal failed on.
- **`inspect_mode` stays.** The §7.6 grouping the map suggested does not exist as
  *state*: `inspect_mode` is one `bool`, and its apparent neighbours —
  `hovered_inspect`, `inspect_snapshot`, `collect_inspection`, `inspect_styles` —
  never read it. They read the tree, the retained boxes and the computed styles,
  every one of which rule 2 forbids an entity from holding, so a "DevTools
  overlay" entity would be one flag plus a wide accessor surface.

Five clusters have now failed the invariant test, and two of them failed inside a
cluster that partly passed. Rule 1 is a per-pair test, not a per-cluster one.

Two things the telemetry extraction made visible and deliberately did **not**
change. `PerformanceSpan` carries a name, a subsystem and an attribution and no
duration, so `open_frame`/`record_phase` emit markers rather than measurements —
`docs/api/12-reality.md` already says so, and the runtime is clock-free by
design, so a duration would have to come from outside it. And a `BudgetViolation`
reaches the **telemetry** sink only: `BudgetViolation::to_diagnostic` exists and
`frame_budget.rs`'s module doc says the diagnostic "flows through the normal
diagnostics channel", but nothing routes it there, so a host that installs only a
`DiagnosticSink` never learns a budget was exceeded. Both are their own change.

### What the editing examination found

The same defect the first three extractions kept turning up — **a doc comment
attached to the wrong item, leaving the real one undocumented** — twice more, and
both are fixed:

- `reseed_editing_from_retained`'s documentation, including the whole "reseed
  discriminator" section that is the only written explanation of the
  controlled-input gate, had `declared_control_value`'s doc glued to its end with
  no item between them. The combined 37-line block landed on
  `declared_control_value`; the method it actually describes had **no doc at
  all**. Split, both verbatim, each on its own item.
- `scroll_paint`'s doc had landed on top of `editing_view`'s, leaving
  `scroll_paint` undocumented.

One behavioural asymmetry is **reported, not changed**: `after_edit` re-derives
both caret-follow axes (`update_caret_scroll` *and*
`update_multiline_caret_scroll`), but `reseed_editing_from_retained` re-derives
only the horizontal one. A controlled `<textarea>` whose value is externally
rewritten shorter therefore keeps a `scroll_top` from the old, longer content,
which can leave the clamped caret scrolled off the top. It is the exact hole the
refused invariant would have closed — worth its own change, with its own test,
rather than riding along in a refactor.

### Why per-container scroll is one field larger than the map

The map counted this cluster as one field, `scroll_top`, and the note beside it
asked the right question: the clamp needs the **live layout**, so either the
extent arrives as an argument or the map stays where it is. Rule 2 answers the
first half — `ScrollOffsets` takes a `ScrollContainer` and never resolves one —
but a map plus an argument is not yet an invariant, and on its own this would
have been a refusal.

What makes it an entity is a field the map's own row did not list.
`folded_boxes` + `folded_boxes_active` are not a neighbouring cluster; they are a
**cache of `scroll_top`**. Each box in the projection is its content-space box
translated up by the summed offset of every scroll container above it, so every
write to the map invalidates it, and the pairing rule 1 asks for is exactly
"`set_scroll_top` must write the offset *and* rebuild the projection". That was
written out by hand — `self.scroll_top.insert(…)` followed by
`self.refresh_scroll_folded_boxes()`, twice, with the same three-line comment
copied above each — and the failure it guards against is silent: a geometry
observer reads the position the node was painted at *before* the scroll.

`set`/`shift` now take the `FoldInputs` they need to refold and do it in the same
call, so that pair cannot come apart. Two writers genuinely cannot: a virtual
list reset to the top by a model replacement, and a fixed-height `<textarea>`'s
caret-follow, both of which clamp against a measure the live `ScrollContainer`
does not carry (the wrapped text's height, for a box that has not been laid out
yet). They go through `seat`, which is named for it and documents that both run
inside an operation that ends in a frame — and a frame refolds.

The prize rule 1 promises showed up too. Eight unit tests now drive scrolling
with **no runtime at all**, including three the old shape could not reach: that
an over-scrolled offset survives a shrink and is restored by the regrowth (the
clamp is on read, not on write, and that is the reason), that a nested scroller
folds its ancestor's offset but not its own, and that a cyclic parent chain
bounds out rather than hanging the frame that publishes geometry.

Two defects, both of the family this campaign keeps finding, and one behaviour
reported rather than changed:

- **`scroll_top` had no doc comment at all.** The ledger above records its doc
  having migrated onto `dump_virtual_geometry` and being put back; what went back
  was the *method's*. The field stayed bare between two documented neighbours,
  which is the same missing blank line seen from the other side. It is documented
  in its new home.
- **`clamped_scroll_top_of` said "clamped to what it can actually scroll"** and
  did not: its body only floors at zero, because it is the read for a caller with
  no live container to clamp against. The claim is now the honest one, and the
  method is named `stored` so the weaker guarantee is visible at the call site.
- `set_scroll_top` rejects a non-finite `px`; **`scroll_by` never checks `dy`**,
  and `(current + NaN).clamp(0, max)` is `NaN` in Rust rather than a panic — so a
  `NaN` wheel delta stores `NaN` in the map, and it reaches `ScrollPaint` and the
  folded projection from there. Preserved exactly as it was: a refactor that
  silently changes what a `NaN` does is a refactor that cannot be reviewed. It is
  its own change, with its own test.

### Why the virtual-list invariant is not the one it looked like

The obvious candidate was *a realized window and the spacers that make its
container's scroll range true must agree* — a window realized without its spacers
sized makes the scrollbar lie. That reads well and it is not this entity's
invariant, because the spacer's extent is **not one of these seven fields**: it
rides on the realized tree nodes, written by `obscur-view`'s
`realize_virtual_lists`, and `size_virtual_spacers` reads it back off the tree and
touches no runtime map at all. An entity cannot guard a pairing whose second half
lives in another crate's output. That is rule 1 rejecting a plausible sentence for
lack of an operation that touches two of *its* fields.

The pairing that does exist is one field over: a list's **window**, the **row
extent** it was realized at, and the **container** it realized into. One operation
records all three — the realize closure writes the range into one map and the
extent into another, side by side, because `index_at` recovers a click's model
index by dividing the offset into the container by that extent. Written apart, a
window recorded without its extent makes `index_at` `continue`: every row of that
list stops responding to clicks *and* to long presses, with nothing reported
anywhere. That is the silent failure rule 1 asks for, and it is why the click and
hold handler maps belong with the geometry rather than beside the other handlers
in `HandlerRegistry` — a virtual row's handler is only reachable *through* the
geometry.

The second pairing is an **ordering**, and it is why `realize` takes
`&mut ScrollOffsets` rather than reporting what to reset. A list whose scroll-reset
token changed has had its content replaced, so its container's offset belongs to
the old content and must be zeroed *before* the window is computed from it. That
was written out by hand in `run_frame` with a comment in capitals saying so. The
`SurfaceGeometry` precedent — report the fact, let the runtime act — would have put
the two calls back in the caller's hands and left the ordering as a comment again;
handing the map in keeps them one call, and the map is still *handed in*, not held.

`size_virtual_spacers` moved anyway, as an associated function over a tree and a
style map. It shares no field with the rest — it is there because it finishes what
they start, and the type doc says outright that this half is stated rather than
guarded.

The prize showed up: **eleven** unit tests now drive virtualization with no
runtime, including four the old shape could not reach — that one `realize` records
a window *and* an extent that resolves a point to a row, that a changed token zeroes
the scroll and the window computed in that same call starts at zero while an
unchanged token leaves both alone, and that a row is offered to the hold recognizer
only where its list registered a handler.

`showcase_png` proves nothing here for the same reason it proved nothing for
`ScrollOffsets`: the showcase declares no `<VirtualFor>`, so none of this code
runs in it. The scene-path proof is the sixteen virtual-list integration tests —
`virtual_list`, `virtual_scroll_reset`, `virtual_row_click` and
`spec_ch4b_virtualization` — which assert realized windows, built row counts,
content extent, scroll-reset behaviour and click/long-press index resolution, and
all still pass.

Two more of the campaign's recurring defect, which brings the tally to **ten**
(the table at the top of this document is the whole of it):

- **`run_frame` had no doc comment.** Its eighteen-line documentation — the only
  written account of why the frame classifies a `FrameReason` into a `FramePlan`
  and skips phases — had merged into the head of `virtual_window_len`'s, so the
  window-length helper's docs opened by describing the frame pipeline and
  `run_frame` itself carried nothing. Split; `run_frame` keeps its own.
- **`size_virtual_spacers`' doc opened with nineteen lines about replaced
  elements.** This is the far side of a fix an earlier stage already made: the
  `size_replaced_elements` doc it names was copied into `ResourceBridge`, where it
  is verbatim and correct, but the original stayed glued above the spacer doc.
  Removed here, where it was a duplicate of text that has a home.

One oddity removed rather than carried: `dump_virtual_geometry` was called as
`let () = self.dump_virtual_geometry();` — the residue of the `#[must_use]` that
the same misplaced-doc defect had dragged onto it, recorded in the ledger above.
The attribute is long gone; the binding it forced is now gone too.

### Why the sheets are one entity and the locale is not

The map counted this cluster as four fields and marked the fourth with a
question. The answer is that it is three: `stylesheets`, `retained_author_sheets`
and `default_stylesheet` are three **sources of one list** — the ordered
`&[Stylesheet]` the matcher runs on — and `locale` is not a source of anything
they produce.

The operation rule 1 asks for is *resolving that list*, and it touches two of the
three in one pass: it substitutes the retained author set when the frame supplies
none, remembers the set when it does, and prepends the UA sheet below whatever
came out. It was written as two statements **190 lines apart in one method** —
retention at the top of `run_frame`, the prepend down inside the style phase —
and the second statement is the only consumer of the first, so nothing between
them explains why they are apart. (They are apart for a reason, and it is now
written down: a frame that skips the style phase must still *remember* the sheets
it was handed, because the next bare frame reads them. Only the remembering has
to happen early; the prepending does not.)

Written apart the failure is silent in both directions:

- **The prepend without the retention** is the defect `retained_author_sheets`
  was added for: a scroll or a hover re-rendered with `&[]` restyles to the UA
  default alone and the author's layout collapses for a frame.
- **The retention over the cascade list** is the one no test named. If what is
  remembered is the list the matcher ran on rather than the author half of it,
  the remembered copy carries the UA sheet — and then **outlives its
  replacement**. A later `set_default_stylesheet` installs a new UA sheet, the
  next bare frame cascades new-UA *and* old-UA, and the stale one wins every tie
  it shares with the new one by sitting later in source order. The one sheet in
  the cascade a host can reinstall becomes the one it cannot.

That second one is why this entity has a companion type. `AuthorSheets` is a
token with no accessor: `CascadeSheets::cascade` takes one and nothing else, and
`author_for_frame` is the only thing that mints one. There is therefore no route
that prepends twice and no route that remembers a prepended list — the ordering
is carried by the types rather than by the comment that used to carry it. It is
the same move `FoldInputs` makes for `ScrollOffsets`, one argument rather than
one return.

The third field earns its place by being the third source rather than a fourth
concern: the reactive path *supplies* its held set to the frame exactly as a
`&View`-taking caller supplies its own slice, so it goes through the same
retention — with the consequence recorded below.

**`locale` fails rule 1 and stays.** It is one `LanguageTag`, and the only thing
`set_locale` touches it with is the scene coordinator, which it pushes the
language subtag into — a coordinator rule 2 forbids an entity from holding, so a
`Locale` type would be a clone and a getter and would move no state. The pairing
that genuinely exists for it (a locale switch must fall through to a **full**
layout, because a language-sensitive substitution can be a different width) is
already owned elsewhere: the language is a member of `LayoutInputs`, so the next
frame's classifier sees the mismatch. Six clusters have now failed the invariant
test, and this is the second time one failed *inside* a cluster that partly
passed.

The prize showed up in its usual form: **nine** unit tests now drive sheet
resolution with no runtime at all, and they assert things that were previously
unobservable from anywhere — the composed list was a local variable inside
`run_frame`, so the only way to ask "what did that frame cascade?" was to render
and read the computed styles back. Four of them could not be written before: that
the UA sheet does not accumulate however many bare frames follow a styled one,
that a replaced UA sheet is replaced on a bare frame too, that an empty set does
not clear the retained one, and that a runtime with no sheets at all cascades
nothing.

`showcase_png` proves this step the way the ledger's earlier steps were proved:
`showcase_hidpi.png` renders byte-identical (`d1e04c9b…`) to the artifact the
previous stage left behind, which is a real proof here rather than a formality —
the showcase installs a UA sheet *and* author sheets and runs the cascade this
step rewrote.

Two behaviours **reported, not changed**, both found by reading closely enough to
move the code:

- **`set_stylesheets(vec![])` cannot turn styling off.** An empty set is how a
  frame says "I own no styling", so the retention takes it as exactly that and
  re-presents the last non-empty author set — forever. A host that means "drop
  the author styling" has to install a sheet with **no rules**. The two meanings
  collide in one value, and the collision is now documented on
  `CascadeSheets::author_for_frame` and covered by a test that names the
  behaviour rather than endorsing it. Giving the host a way to say the second
  thing is its own change.
- **`set_default_stylesheet`'s "any other origin still works" is true only at the
  top level.** The doc says only the matched *order* is fixed and that precedence
  comes from the origins on the rules. Inside a component that is not so:
  `scoped_internal_declarations` exempts the default sheet from §4.15 scoping by
  testing `origin == CascadeOrigin::SystemDefault` exactly, so a default sheet
  installed at any other origin is scoped **out** of every component-internal
  node while still applying to top-level ones. The claim and the matcher disagree
  about a case a host can reach; which of the two is wrong is a decision, not a
  refactor.

This is also the first extraction in the campaign that found **no** misplaced doc
comment. All three fields and all three public methods carried their own docs on
their own items, so the count of that recurring defect stays at ten.

### Why the a11y half is an entity and the drag half is not

The map's last row lumped two concerns together because both were small. They are
unrelated, and rule 1 — a per-pair test, not a per-cluster one — answers them
differently.

**Accessibility passes.** `accessibility` and `published_focus` are read together
by one operation, *publishing*, which decides whether to push by asking whether a
bridge exists **and** whether focus moved since the last push, and which then
reports the node focus moved *from* and records the node it moved *to* in the
same call. The second half is the interesting one, because `published_focus` is
not a fact about the runtime at all: it is a fact about the **port** — "what has
this bridge already been told". Written apart the failure is silent both ways:

- **A push that forgets to record.** The gate `published == focused` never goes
  true again, so every frame after it re-walks the whole retained tree and
  re-pushes the same snapshot — exactly the cost the gate exists to avoid, paid
  at frame rate with nothing on screen to show for it. The next push also names a
  `previous` node focus already left, so an adapter following focus movement is
  told a transition that did not happen.
- **A record without a push.** The gate then suppresses the push that would have
  carried the move, and the platform is never told focus changed at all.

Rule 2 shapes the split the way `ScrollOffsets` was shaped: the publisher never
sees the retained tree, so it cannot build the snapshot, and `should_publish` is
therefore a *separate question* from `push` rather than a branch inside it — the
gate has to be answerable before the full tree walk it guards is paid for.

The prize showed up: **eight** unit tests now drive the §7.1 output path with no
runtime at all, and four of them could not be written before, because the only
way to ask "did that frame push?" was to install a bridge through a whole runtime
and count `tree_updates` — that a focus move publishes exactly once and the three
idle frames behind it publish nothing, that each push names the node the one
before it moved to, that clearing focus is a move like any other and stays
published, and that the order inside a push is focus-then-tree (an adapter
converts a tree against the focus target it was last *told*, so a snapshot pushed
first is converted against a stale one).

`showcase_png` proves nothing here and the reason is the clean one:
`publish_accessibility` sits on the frame path but produces no display list, so
this is the `FrameTelemetry`/`BackgroundWork` case. The proof is the suite —
`accessibility_output_path` (5), `spec_ch7_accessibility` (1) and the runtime's
own `push_accessibility_feeds_the_bridge` — which assert what reaches a real
`AccessKitBridge`, and all still pass.

**Drag fails, and it fails the way `TextEditor` did.** The row counts two fields,
but one of them *is already an entity*: `DragCoordinator` has its own file, its
own invariant (a staged `ProvisionalTransaction` that a cancel consumes, so a
cancelled drag cannot later commit) and its own eight unit tests. What is
left on `UiRuntime` that is the cluster's own is exactly **one** field,
`drag_outcome` — the same count that refused the editing cluster.

And `drag_outcome` is not state the drag reads. `begin`, `pointer_moved`,
`cancel` and `release` never consult it; it is a one-slot mailbox the host drains
through `take_drag_outcome`. Only one operation touches both fields —
`finish_drag`'s last statement — and the three tests the `TextEditor` proposal
failed are failed again:

1. **It moves one field of state, and the one that moves is a handle to an entity
   that already has a home.** `DragCoordinator` + `Option<DragOutcome>` → a
   wrapper is 48 → 47 on the same measure as the accessibility step, but what
   would move is not "state and the logic that guards it": the logic is
   `release`'s, and `release` is already in `drag_coordinator.rs`.
2. **It can own only a third of its candidate invariant.** Ending a drag must
   park the outcome, **apply the drop's commit** and **schedule a frame**. Rule 2
   forbids the entity holding the enactor or the scheduler, so it would own the
   parking and report the other two — and the parking is the least consequential
   of the three. Forget it and the host learns nothing; forget either of the
   others and the drop does nothing, or the drag's visuals never leave. The
   runtime is still the thing that must remember, which is where enforcement
   evaporated last time.
3. **Nothing becomes testable that is not already.** The lifecycle is already
   drivable with no runtime through `DragCoordinator`, and the runtime path is
   covered by `drag_and_drop` (7) and `spec_ch6_data_transfer`. What is left is
   `take()` emptying a slot.

Every other invariant this campaign has accepted has a second *read* that can
disagree with the first — the folded boxes against the offsets, the retained
sheet against the UA sheet, the pending count against the serviced total, the
gate against the record. `drag_outcome` has no second reader to disagree with
anything. Seven clusters have now failed the invariant test, and this is the
third time one failed inside a cluster that partly passed.

Two behaviours **reported, not changed**, and one hazard found next door:

- **One record, two push routes.** `push_accessibility` is public "for a host
  that drives a bridge of its own instead", and it advances the same
  `published_focus` the installed bridge's gate reads. A host that drives its own
  bridge *as well as* installing one therefore satisfies the gate with its own
  push, and the installed bridge misses that focus move unless the tree changed
  too. The two routes are alternatives, and one record is what makes them so;
  documented on `AccessibilityPublisher::push` and covered by a test that names
  the behaviour rather than endorsing it.
- **Installing a bridge does not reset the record.** The immediate push that
  makes installation order not matter reports a `previous` belonging to the
  bridge *before* it — and where focus has not moved since, reports a change from
  a node to itself, which `AccessibilityFocus::changed` reads as no change at
  all. `AccessKitBridge` acts on `focused` and is unaffected; an adapter that
  gates on `changed` would never learn the focus of the tree it was just handed.
  Which of the two readings of the port's contract is right is a decision, not a
  refactor.
- **`DragCoordinator::release` is not `#[must_use]`**, and `Option` is not
  `#[must_use]` as a type (checked, not assumed). Dropping its return therefore
  loses a validated `TransactionCommit` with no warning anywhere — the one way
  the pairing above could silently come apart, and a one-attribute change that
  belongs in its own commit rather than riding along in a refusal.

This is the second extraction in a row that found **no** misplaced doc comment,
so the count of that recurring defect stays at ten. Both fields and all five
methods that touch them carried their own docs on their own items.

## The guard

`tools/xtask/src/rule/file_size.rs` caps a source file at **2,000 lines** and
carries a ledger of the ten files over it, each with its reason. The cap is taken
from the workspace it governs — 1,671 of 1,695 files are already under 1,000
lines — so it catches a second entity forming, not ordinary work. The rule fails
both ways: a new oversized file with no entry, **and** a ledger entry whose file
has since come under the cap, so a shrunken file cannot grow back through the
hole its entry left behind. Both directions were exercised before landing.

---

# The second pass: the behaviour seam

**Status:** fourteen entities extracted along a *different* seam from the first
twelve, and the result is deliberately **reduced, not under**. `ui_runtime.rs`
went from **7,272 → 5,548 lines of non-test code** (17,372 → 15,514 total). The
owner's operational bar is 1,000 lines of non-test code; this pass does not reach
it, and the last section of this document argues why no honest extraction can,
rather than treating the shortfall as unfinished work.

Measured on the tree as it stands: suite **6,121 passed / 0 failed** (from 6,100
— the 21 are the new files' own unit tests, and no existing test was weakened,
rewritten or deleted), `clippy` **388** workspace warnings on a clean build (from
391), `fmt` clean, `xtask check` and `xtask consumer` both pass.

## Why the seam had to change

The first pass asked "which **fields** share an invariant?" and that question is
exhausted: it drove the field count from 103 to 47 and its own ledger says every
cluster on the map has been extracted or refused with reasons. What it could not
touch is that the file was still **7,272 lines of behaviour** in one
`impl UiRuntime` — because a method's home was decided by which fields it read,
and the remaining 47 fields are all read by the frame pipeline.

So this pass asks the other half of spec §3.16's question. The rule is *"один
файл не экспортирует несколько главных сущностей"* — one file must not export
several main entities — and §3.1 spells out what an entity is: one main entity,
its `impl`, **private local functions with no independent semantics**, and its
unit tests. The operative words are the last five. A private method that
implements a self-contained sub-algorithm — one with its own vocabulary, whose
interface is far narrower than its internals — *has* independent semantics, and
§3.1 does not admit it into another entity's file merely because it happens to
take `&self`.

Read that way `ui_runtime.rs` was not one entity plus helpers. It was one entity
plus a hit-test resolver, a DevTools projection, a component-render protocol, a
command-dispatch algorithm, a restyle phase, a layout classifier and five value
types — and three of those value types (`LayoutInputs`, `StyleKey`, `StyleDelta`)
plus `FramePhaseClock` were *literal* second exports of the file, which is the
§3.16 violation in its plainest form. `FramePhaseClock` and `restyle_roots_of`
were even declared **below** the 10,000-line test module, which is how invisible a
second entity can become in a file this size.

## The rules this pass follows

The first pass's four rules still hold, and this pass adds one and reads one
differently.

1. **A group becomes an entity when its interface is much narrower than its
   internals.** This is the behaviour analogue of rule 1's invariant test.
   `HitTesting` is one call and four sort terms; `ComponentRenderPass` is one
   call and a three-answer error-boundary protocol; `CommandDispatch` is three
   calls over a four-step propagation and a five-link chain.
2. **The entity takes what it needs as arguments** — unchanged, and pushed
   further. Most of what came out this time owns *no* state at all, because the
   state it reads is the frame pipeline's. Those types are **borrow bundles**:
   `struct XPass<'a> { tree: &'a RetainedTree, … }`, constructed for one phase and
   dropped with it. That is rule 2 taken to its limit rather than a departure from
   it — the alternative is a six-argument free function whose arguments nothing
   names.
3. **The public API does not move** — unchanged, and it is now the binding
   constraint. Every `pub fn` kept its signature and its doc comment on
   `UiRuntime`; only bodies delegate.
4. **A method with no production caller is not created** — unchanged.
5. **Report, do not act.** An extracted phase that would have to hold the
   scheduler, the diagnostics sink or the offset map instead *returns* what it
   decided and the runtime applies it: `TreeInspector::set_mode` returns whether
   the toggle was an edge, `CommandDispatch::dispatch` returns a
   `CommandDispatchOutcome` the runtime turns into a frame request, and
   `CaretScroll` returns an offset rather than seating it. This is the
   `SurfaceGeometry` precedent from the first pass, applied to behaviour.

## Done

| Entity | Non-test lines | What it is |
|---|---|---|
| `runtime/hit_testing.rs` — `HitTesting` | 334 | The z-order index and the stacking-order resolution over it. Owns `spatial`, which is why it is the one extraction here that also moves state |
| `runtime/tree_inspector.rs` — `TreeInspector` | 325 | §7.6's DevTools projection: the tree snapshot, the hovered readout, the highlight overlay, and the flag that arms it |
| `runtime/restyle_phase.rs` — `RestylePhase` | 360 | Frame phases 2–3: the restyle scope, the full pass, the incremental pass, and the reconcile→restyle-roots mapping |
| `runtime/command_dispatch.rs` — `CommandDispatch` | 311 | §2.3's four propagation steps and §2.6's five-link routing chain, which are the same thing |
| `runtime/component_render_pass.rs` — `ComponentRenderPass` | 307 | Frame step 1c: the bounded render fixpoint and the §1.7 error-boundary protocol |
| `runtime/layout_classifier.rs` — `LayoutClassifier`, `LayoutPhase` | 266 | Step 8's classifier: the conservatism ladder plus the per-node layout-affecting diff |
| `runtime/caret_scroll.rs` — `CaretScroll` | 261 | Caret-follow scroll-into-view, both axes, and the field-scrollport arithmetic they share |
| `runtime/scroll_targets.rs` — `ScrollTargets` | 174 | The frame's live scroll containers: which one a point addresses, the point in unscrolled space, and each one's paint state |
| `runtime/editable_field.rs` — `EditableField` | 124 | What the retained tree says about a field, as against what the session says |
| `runtime/frame_phase_clock.rs` — `FramePhaseClock` | 104 | `OBSCUR_FRAME_STATS` per-phase timing — a second entity that was sitting *below* the test module |
| `runtime/ancestor_chain.rs` — `AncestorChain` | 66 | The cycle-bounded logical ancestor walk five call sites had open-coded |
| `runtime/style_key.rs` — `StyleKey` | 41 | Everything a retained style map was produced from |
| `runtime/layout_inputs.rs` — `LayoutInputs` | 37 | What layout reads that is not a computed style |
| `runtime/style_delta.rs` — `StyleDelta` | 34 | What the style phase leaves the layout classifier to compare against |

Fourteen files, **2,744 lines of non-test code** and **21 new unit tests**. The
arithmetic does not close the way the first pass's field count did, and that is
the honest shape of a behaviour seam: what a public method leaves behind is its
doc comment and a delegating body, so 2,744 lines moved out reduce the file by
1,724.

### What the prize looks like this time

The first pass measured its win in fields. This one measures it in what became
reachable without a runtime. All 21 new tests construct their subject directly:
an empty `HitTesting` resolves nothing, a `TreeInspector` reports only real
toggle edges, a route with no origin is exactly the two non-positional links, a
`LayoutClassifier` with nothing retained lays out in full, a `CaretScroll` over a
field with no text leaf answers `None` on both axes rather than resetting it to
zero, and `ScrollTargets` over a container-free frame folds a point to itself —
the property that keeps a non-scrolling document's hit-test byte-identical to one
built before scroll containers existed. Every one of those took a whole
`UiRuntime`, a view and a frame to observe before.

### What this pass found

Three things, and two of them are the campaign's recurring defect in a form the
first pass's own extractions left behind.

| # | Where | What |
|---|---|---|
| 11 | the `pointers` field | The doc of the **removed** `presses` field was still glued above `pointers`'s own doc with no blank line, so `rustdoc` read 12 lines about click anchors and per-finger keying as the opening of the pointer-sessions doc. Its subject moved to `PointerSessions` in the first pass but its text did not: restored to `PointerSessions::presses`, where the "keyed rather than single" rationale had been lost entirely |
| 12 | the `routing` field | The same shape: the removed `commands` field's 22-line doc glued above `routing`'s. The half with no other home — that a `UiRuntime` *is* one window surface, so a handler on the runtime is a link-4 handler by construction, and that link 4 is a deterministic single lookup — moved to `HandlerRegistry::handlers`; the rest was already on `on_command`, where a caller reads it |
| — | `handle_scroll` | Open-coded `scrollable_container_at` **exactly**: the same filter, the same empty check, the same fold. Not a near-duplicate — the same eight lines twice, one of them public and one private. Both now call `ScrollTargets::scrollable_at` |

The count of the misplaced-doc-comment class therefore stands at **twelve**, and
the two new ones are worth their own note: they were created *by* an extraction,
not merely found by one. When a field is replaced by a handle to an entity, its
old doc has to travel with the state it described — otherwise it stays behind and
silently becomes the first paragraph of its replacement's.

Six intra-doc links to moved items were re-pointed; `cargo doc` reports no
warning against any file this pass created or changed that it did not report
before.

## Why 1,000 lines is not reachable, and what the number would cost

The bar is 1,000 lines of non-test code. The file is at 5,548. That gap is not
five more extractions away, and the reason is arithmetic rather than appetite.

What is actually in those 5,548 lines, measured:

- **~200 public methods of `UiRuntime`'s own API**, averaging ~22 lines each and
  overwhelmingly *doc comment* rather than body. Rule 3 pins them here: a caller
  reads `run_due_frame`'s contract on `run_due_frame`. Moving the docs is not a
  refactor, it is a documentation regression, and moving the methods is an API
  break.
- **`run_frame`, 578 lines**, and **`handle_pointer`, 331**. These are the two
  places the whole thing is actually integrated. `run_frame` reads or writes
  every one of the 47 fields; `handle_pointer` touches fifteen and calls twenty
  of the runtime's own methods. A borrow bundle for either would name almost the
  entire struct, which is not an entity — it is the struct with an extra
  indirection. The first pass's ledger already says this in its own words: what
  is left "is the reconcile → style → layout → compose sequence … and every one of
  them is read by the pipeline methods that stayed."
- **~600 lines of struct, field docs, imports and the `Debug` impl.**

Add those and the floor for this file, without breaking rule 3, is somewhere near
**4,000–4,500**. The 5,548 it sits at is roughly a thousand lines above that
floor, and those thousand are the remaining mid-sized clusters named below — each
a real extraction, none of them enough to matter against the bar.

The bar is therefore met the only way it can be: by saying what it would cost.
Reaching 1,000 requires deciding that `UiRuntime`'s public surface should be
several types rather than one — an API change, not a file split. That is a
worthwhile question and it is a *design* question; answering it inside a
refactoring pass, silently, would be exactly the kind of change this campaign's
method exists to prevent.

## What is left, and what it is worth

Named so the next reader inherits the list rather than re-deriving it:

- **The text-edit key resolution** — `try_text_edit` (151 lines),
  `try_vertical_move`, `grapheme_cluster_range`, `line_boundary_move`,
  `visual_line_boundary`: ~300 lines that turn a `KeyEvent` into an `EditCommand`
  against a laid-out field. A real entity; entangled with `apply_edit`/`after_edit`
  in ways that want their own change.
- **The pointer gesture arming** — `arm_pan`, `arm_long_press`, `advance_holds`,
  `fire_long_press`, `launch_fling`, `advance_flings`: ~290 lines that are already
  thin over `PointerSessions`, whose recognizers the first pass made pure. The
  win here is smaller than it looks.
- **The editing orchestration** the first pass refused on state grounds
  (`sync_editing_to_focus`, `reseed_editing_from_retained`, `commit_editable_change`,
  `begin_text_pointer`, `update_text_drag`, `select_word_at`, `select_line_at`).
  The refusal was about *state* and it still stands; whether these have
  independent *semantics* is a live question this pass did not have to answer,
  because the caret geometry and the tree-side field queries — the two halves that
  clearly did — came out on their own (`CaretScroll`, `EditableField`).

None of the ten behaviours the first pass **reported and did not change** were
touched by this one. They are still open, still documented where they live, and
still covered by tests that name the behaviour rather than endorsing it.

---

# The third pass: the compose phase, along the painter seam

**Status:** done. **Sixteen entities extracted**, and
`orchestration/scene_coordinator.rs` went from **4,281 to 800 lines of non-test
code** (9,550 → 5,831 total). Suite 6,121 passed / 0 failed — unchanged, with no
test weakened, rewritten or deleted; `clippy` 387 → 386; `fmt` clean; `xtask
check` and `xtask consumer` both pass.

## Why the seam is different again

The first two passes cut `UiRuntime` along **state** and then along
**behaviour**, because a god object with 103 fields is a dozen entities wearing
one struct's name. `SceneCoordinator` is not that: it had **three fields**, all
three of them the text shaper, and the rest of the file was **free functions**.
Rule 1 — "a cluster becomes an entity only where its fields share an invariant" —
had almost nothing to bite on.

What the file actually held was **painters**: `background_color_of` →
`background_image_geometry` → `paint_background_layer`; `border_fills` →
`uniform_border_ring` → `dashed_border_path`; `color_op_matrix` → `mat_mul` →
`color_matrix_of_ops`. Each group is a small vocabulary answering one question
about a node — what brush fills it, what path strokes its edges, what 4×5 matrix
its filter chain folds to — and each has the asymmetry that marks an entity: a
handful of call sites in the walk, and a hundred-odd lines of CSS behind them.
So this pass tests a different rule:

> **A group is an entity when its interface is much narrower than its internals,
> and its internals speak a vocabulary the caller never has to learn.**

`border_paint` is fourteen functions the walk calls three of. `gradient_brush` is
eight the walk calls none of — only `background_paint` does. That ratio is the
seam; the line count was only the symptom that let us find it.

## Done

| Entity | Lines | What it owns |
|---|---|---|
| `orchestration/paragraph_shaping.rs` — `ParagraphShaping` | 601 | The phase's **only** state: the paragraph orchestrator, the default fallback face and the hyphenation port. They are one entity because the same engine must shape a run in the measure pass and in the compose pass — swapping the shaper without dropping the memoized runs, or hyphenating in one pass and not the other, makes a measured box height and a painted line count disagree |
| `orchestration/caret_geometry.rs` | 537 | The byte ↔ paragraph-geometry mapping: the caret's pen x at a byte, the byte nearest an x, the line a byte sits on, vertical motion, Home/End, the scroll extent. Every one of them walks the **same spaced/tabbed advances the segments are painted from**, which is the whole reason a click resolves back to the offset whose caret was drawn there |
| `orchestration/text_painter.rs` | 442 | A text node's emitted paint: its runs per wrapped line, and — for the focused editable — the selection highlight behind the glyphs, the IME preedit underline and the caret on top. Plus `::before`/`::after`/placeholder runs, which paint into the same unit |
| `orchestration/subtree_brackets.rs` — `SubtreeBrackets` | 433 | **A bracket is one fact with two ends.** A `PushClip` is meaningless without its `PopClip`, the pop cannot be emitted until the walk has seen the subtree's last descendant, and the relative nesting of the six kinds — blur outside transform outside group-layer outside clip, scroll transform inside it — is a single decision. The walk used to declare eight `Vec`s, push into them, and then decide the nesting **700 lines later** |
| `orchestration/border_paint.rs` | 453 | The stroked edges of a box: the mitered border quads, the uniform rounded ring, the dashed stroke, the resolved corner radii every other consumer of the box shares, and the `outline` ring outside it |
| `orchestration/background_paint.rs` | 293 | `background-color`, each `background-image` layer, and the `background-size` / `-position` / `-repeat` tile geometry a resolved image draws with |
| `orchestration/gradient_brush.rs` | 271 | CSS Images 3 gradients → scene fill brushes: the linear gradient *line* (including the aspect-ratio-aware `to <corner>` direction), the radial ending shape, the conic sweep |
| `orchestration/filter_color_matrix.rs` | 181 | A `filter` chain's colour functions folded into ONE 4×5 affine matrix. Pure algebra with its own vocabulary (`mat_mul`, `mat_vec`, `add4`), and the only place the spec's luma/sepia/hue-rotate constants live |
| `orchestration/replaced_content.rs` | 178 | Placing a replaced element's own content — bitmap or vector document — inside its box by `object-fit` / `object-position` |
| `orchestration/composite_effects.rs` | 137 | The group-layer effects a node composites its whole subtree through: `opacity`, `mix-blend-mode`, `filter`, `backdrop-filter` |
| `orchestration/editing_role.rs` — `EditingRole` | 110 | Whether and how the live session buffer overrides a text node's text (the CSS `::placeholder` / `:placeholder-shown` model). Already shared by the compose **and** the measure pass, and never belonged to either |
| `orchestration/scroll_container_paint.rs` | 108 | What a scroll container contributes: whether its `overflow` clips, and the scrollbar track + thumb in the scrollport gutter |
| `orchestration/box_shadow_paint.rs` | 75 | A node's `box-shadow` list and the even-odd clip that keeps a **drop** shadow outside the border box (css-backgrounds-3 §6.1) |
| `orchestration/paint_transform.rs` | 73 | A CSS 2D `transform` resolved about its `transform-origin` into the subtree affine |
| `orchestration/paint_color.rs` | 46 | `currentColor`, the text ink, and the authored-to-scene colour conversion every painter shares |
| `orchestration/scroll_paint.rs` — `ScrollPaint` | 29 | A second **exported** entity that was living in the coordinator's file. §3.16 forbids exactly that, whatever its size |

Sixteen files, **3,967 lines**. What is left in `scene_coordinator.rs` is the
pre-order walk itself, the three `compose*` entry points, one field, and the
seven public setters whose doc comments must stay where a caller reads them.

## What did not move, and why

- **The walk.** It is `SceneCoordinator`'s reason to exist: visit each node in
  pre-order, ask each painter what this node contributes, capture the answer as
  one `PaintUnit`, sort. Every remaining line is that sentence.
- **The public API.** `compose`, `compose_with_images`,
  `compose_with_images_scrolled`, `paragraph`, and the five setters keep their
  signatures **and** their doc comments; only their bodies delegate. `Debug` now
  prints one nested `text` field instead of three flat ones — the same
  information, one level down, and nothing asserts on it.
- **The tests.** 5,031 of the file's 5,831 lines are its `#[cfg(test)] mod
  tests`, and they belong there: they drive the public `compose*` API and assert
  the emitted display list. Only **ten** tests moved — the four gradient-geometry
  tests, the five colour-matrix tests, and the `EditingRole` classification test —
  because those drive an extracted entity **directly** rather than through
  `compose`. Each moved with its assertion untouched.
- **Crate-internal caret calls did move.** `caret_advance_x`,
  `caret_offset_at_point`, `caret_line_move`, `caret_visual_line_bound` and
  `caret_line_scroll_geometry` were `pub(crate)` methods on `SceneCoordinator`;
  they are now `pub(crate)` functions of `caret_geometry` taking the engine
  explicitly (campaign rule 2), and their five call sites in `ui_runtime.rs` and
  `caret_scroll.rs` read `caret_advance_x(self.composer.text(), …)`. Their doc
  comments travelled with them, so a caller still reads them where it calls them.
  This is a crate-internal move, not a public API change.

## What this extraction found

**One instance of the campaign's recurring defect** — the eleventh — and it is
the same shape as the other ten: two contiguous `///` blocks with the blank line
missing between the items they belong to.

| Found by | The item left bare | Where its doc had gone |
|---|---|---|
| `ParagraphShaping` | `shape_text_lines` | `align_width_for`, whose documentation therefore opened by describing the *shaping* function and only then reached the alignment width it is actually about |

The whole production half of the file was then swept for the symptom — an item
with no doc comment at all — and `shape_text_lines` was the only real hit (the
other two are a `Default::default` and a function-local `const`). Recorded so the
next reader knows the sweep happened rather than assuming it.

## What is left, and what it is not

`scene_coordinator.rs` stays on the `file_size.rs` ledger, because the ledger's
cap counts **total** lines and 5,031 of the remaining 5,831 are tests. Under the
operational bar this campaign is actually held to — 1,000 lines of non-test code
— it is done at **800**.

The next reduction is not another painter. It would be splitting the test module
by concern, which spec §3.1 argues *against* ("that entity's unit tests" belong
with the entity, and these tests' subject is `compose`), and which would need its
own justification rather than this campaign's.

---

# The fourth pass: the layout phase, along the pass seam

**Status:** done. **Nine entities extracted**, and
`orchestration/layout_coordinator.rs` went from **2,835 to 518 lines of non-test
code** (5,113 → 2,451 total). Suite 6,121 passed / 0 failed — unchanged, with no
test weakened, rewritten or deleted; `clippy` 386 → 386; `fmt` clean; `xtask
check` and `xtask consumer` both pass. Under the owner's operational bar of 1,000
lines of non-test code, this file is **done**.

## The seam

`file_size.rs`'s ledger entry named the shape years before this pass ran: *"the
arrange walk plus its style→layout mapping; the mapping is the extractable
half"*. That was half right. The mapping did come out — it is the largest single
piece — but the file was not two entities, it was **ten**, and the other eight
are the reason 518 was reachable where the ledger's note would have predicted
something closer to 1,900.

What the file actually held was **one phase of the frame pipeline written out
end to end**: the entry points, the style→layout bridge under them, the
text-leaf measure that bridge's `font_size` feeds, the `display`→algorithm
dispatch, the pure measure recursion, the absolute-positioning arrange
recursion, two retained caches, and the §4.3 dependency analysis that decides
what those caches may keep. Every one of those speaks its own vocabulary and is
called across a narrow interface — which is the third pass's rule, applied to a
pipeline rather than to a painter:

> **A group is an entity when its interface is much narrower than its internals,
> and its internals speak a vocabulary the caller never has to learn.**

`text_leaf_measure` is seven functions the walk calls **one** of. The keyword
half of the mapping is nineteen readers the coordinator calls **none** of — only
`layout_style_from` does. `layout_eviction` is one call over two tree walks and
a written-down dependency list. The line count was the symptom; the ratio is the
seam.

Two of the ten were also **literal second exports of the file**, which is §3.16
in its plainest form: `MeasureCache` and `ArrangeCache` are `pub`, re-exported
from `orchestration/mod.rs`, and were sitting inside `LayoutCoordinator`'s file
with 400 lines of their own documentation and no relation to the coordinator's
`impl` beyond being handed to it.

## Done

| Entity | Non-test lines | What it owns |
|---|---|---|
| `orchestration/arrange_pass.rs` — `arrange` | 524 | The placing half. Absolute box accumulation, the containing block a positioned **or transformed** node establishes (css-transforms-1 §3), the scroll container's block-axis relaxation, the percentage basis a child inherits (and the grid item's exception to it), `position: sticky`'s scrollport clamp, and out-of-flow placement |
| `orchestration/layout_style_mapping.rs` — `layout_style_from` | 451 | The [`ComputedStyle`] → [`LayoutStyle`] bridge and the nineteen keyword readers under it. Nearly pure: a computed style plus a resolution context in, a layout style out, no tree and no side-tables |
| `orchestration/text_leaf_measure.rs` — `measure_text_node` | 416 | What a text leaf measures to (spec §5.10): shape once, fold in letter-/word-spacing and tab stops, wrap to the offered width. Plus the four single-glyph probes — hyphen, ellipsis, space, `0` — which all answer the same question through the same `measure_face`, and that shared face is what keeps the measured box and the painted glyphs font-consistent |
| `orchestration/arrange_cache.rs` — `ArrangeCache`, `ArrangeKey` | 243 | A **second public entity** that was living in the coordinator's file. The retained subtree, the key it may be served under, and the written argument for why neither the key alone nor the dirty set alone would be sound |
| `orchestration/length_resolution.rs` | 237 | One computed length → layout's numeric vocabulary, and the per-node `LengthContext` that gives `em`/`rem`/`vw`/`ch` their bases |
| `orchestration/measure_pass.rs` — `measure` | 215 | The pure sizing recursion and the `RecMeasurer` that carries it into an algorithm |
| `orchestration/layout_eviction.rs` — `invalidated_layout` | 187 | §4.3: what a `LayoutInvalidation` can have changed, as two sets over two tree walks |
| `orchestration/measure_cache.rs` — `MeasureCache` | 133 | The other **public** entity that was living in the coordinator's file, plus the exact `(node, constraints)` key it memoizes on |
| `orchestration/formatting_context.rs` | 123 | Which formatting context a node establishes, and which children enter it — both `display`'s doing, which is why they are one entity |

Nine files, **2,529 lines of non-test code**. What is left in
`layout_coordinator.rs` is the five documented entry points, the walk they share
(`arrange_all`, which builds the initial containing block and starts the
recursion), the two side-table aliases, and the recursion bound.

## Why the mapping is two files and not one, or three

The mapping is ~690 lines all told, so a single file would have cleared the bar
on its own. It is split because §3.16 asks how many entities a file exports, not
how long it is — and the answer here is objective rather than aesthetic:

- **The length half has a second consumer.** `grid_templates.rs` already
  imported `length_context` and `ch_width_for` out of this file, by name, to
  fold a fixed track's relative units. A helper that another module reaches into
  is not "a private local function without independent semantics" (§3.1); it is
  an entity someone is already using as one. That import now reads
  `super::length_resolution::{ch_width_for, length_context}`, which says where
  it comes from.
- **The keyword half does not.** Each of the nineteen keyword readers compares
  one property's interned symbol against the reserved `layout_keyword` handles
  and is called exactly once, by the assembly above it. That is §3.1's private
  local function, and exiling them to a third file would have been the arbitrary
  split this campaign's method exists to refuse.

The two halves share nothing: no call passes between them, and their import sets
are disjoint (`InternedSymbol` + `layout_keyword` on one side, `Length` +
`LengthContext` + `calc()` on the other).

## What did not move, and why

- **The two side-table aliases.** `LayoutBoxes` and `LayoutStyles` are `pub` and
  re-exported, so on a strict reading of §3.1 they are two more equal public
  entities. They stay: a type alias has no `impl`, no invariant and no unit
  tests, and its documentation is about what the *coordinator* produces and
  consumes. Two fifteen-line files here would be the empty layer §3.1 warns
  against, and would touch every import in the crate for nothing.
- **The public API.** All five entry points — `layout`, `layout_with_hyphenator`,
  `layout_editing`, `layout_invalidated`, `layout_styles_from` — plus
  `grid_templates_from` and the two named-placement resolvers keep their
  signatures **and** their doc comments; only their bodies delegate.
  `crate::orchestration::{ArrangeCache, MeasureCache, root_font_size}` still
  resolve, re-exported from their new homes.
- **The tests.** 1,933 of the file's 2,451 remaining lines are its
  `#[cfg(test)] mod tests`, and they belong there: they drive the public
  `layout*` API and assert absolute boxes, incremental measure/arrange counts and
  cache-serving behaviour. Only **fourteen** moved — the thirteen `mapping_*`
  tests, which call `layout_style_from` directly, and
  `root_font_size_reads_root_node_defaulting_to_sixteen`. Each moved with its
  assertion untouched.

## What this extraction found

**Two instances of the campaign's recurring defect** — the thirteenth and
fourteenth — and both are the same shape as the twelve before them: two
contiguous `///` blocks with the blank line missing between the items they
belong to, so `rustdoc` reads them as one comment on the *second* item.

| # | Found by | The item left bare | Where its doc had gone |
|---|---|---|---|
| 13 | `length_resolution` | `insets_of` | `margin_edges_of`, whose documentation therefore opened with one line about pixel insets and only then reached the margin edges it is actually about — and the irony is that the misplaced line names `insets_of` as the thing `margin_edges_of` is the twin *of*, so the two docs read as one and the reader has no way to see that a function is missing |
| 14 | `measure_pass` | `measure` | `overflow_for_text`, a six-line `text-overflow` lookup whose documentation opened with the whole ten-line account of the pure measure pass — the borrow-cleanliness argument that is the only written explanation of why measure and arrange are separate functions at all. `measure` itself carried nothing |

Defect 14 is the more consequential of the two, and it is the reason this pass's
`measure_pass.rs` module doc can state the borrow argument at the top: that text
existed, correctly written, attached to the wrong item, a hundred lines from
where anyone would look for it.

One pre-existing broken intra-doc link was corrected in passing:
`scroll_container_of` pointed at `super::establishes_scroll_container`, which
does not exist — the item is `super::scroll::establishes_scroll_container`.

Eleven intra-doc links to items that moved were re-pointed. `cargo doc` reports
no unresolved link against any file this pass created that it did not report
before; the one that remains (`GridLayout::new(1)`, in `formatting_context.rs`)
is not a resolvable path in any location and predates this pass.

## What is left, and what it is not

`layout_coordinator.rs` stays on the `file_size.rs` ledger, because that cap
counts **total** lines and 1,933 of the remaining 2,451 are tests. Under the
operational bar this campaign is held to it is done at **518**, and the ledger
entry now says which nine entities left and what the remainder is, rather than
predicting a mapping extraction that has since happened.

The next reduction here would be splitting the test module by concern — which
spec §3.1 argues *against* ("that entity's unit tests" belong with the entity,
and these tests' subject is `layout`) and which would need its own
justification rather than this campaign's.

# The fifth pass: the CSS algorithms, along the spec-section seam

**Status:** done. **Ten entities extracted** from the two layout algorithms the
`file_size.rs` ledger had been defending on the grounds that each is "one
algorithm":

* `algorithm/grid_layout.rs` — **1,911 → 636 lines of non-test code** (4,142 →
  2,867 total);
* `algorithm/flex_layout.rs` — **1,622 → 695 lines of non-test code** (3,180 →
  2,253 total).

Both are now under the owner's operational bar of 1,000 lines of non-test code.
Suite 6,121 passed / 0 failed — **unchanged**, with no test weakened, rewritten,
deleted *or moved*; `fmt` clean; `xtask check` and `xtask consumer` both pass;
`clippy` **374** workspace warnings, and `obscur-layout` is now clippy-clean —
the one warning it carried (a `single_match_else` over `stretch_target`) was
fixed in its new home rather than carried into it. `showcase_png` renders
**byte-identical**: `showcase_hidpi.png` hashes `d1e04c9b…`, the same digest the
third pass recorded, and the showcase drives both algorithms on every frame.

## Why the seam is different again

The first four passes cut a god object along a seam of its own making — state,
behaviour, painter, pass. Neither of these files was a god object. Each
implements one W3C algorithm end to end, and the ledger entries defending them
said exactly that: *"css-grid-2 §11 track sizing is one algorithm and is tested
as one"*, *"css-flexbox-1 §9 is one algorithm with one freeze loop"*.

Both claims are **true**, and neither is a reason to keep 1,911 lines in one
file. §3.16 asks how many main entities a file exports, and here that question
has an unusually objective answer, because the algorithm is not a monolith in
its own specification: **the W3C section headings are the entity list.** Grid's
placement (§8.5) does not measure anything; its track sizing (§12.3) does not
know where an item is; its track positioning (§11.5) is arithmetic over a
resolved `&[f32]`; its item alignment (§11.3/§11.4) reads two style keywords.
Flex's §9.7 freeze loop takes eight numbers and returns a vector, and its §8.1
`auto`-margin rule is two functions that disagree on one input.

So the seam this pass follows is the third pass's rule, applied to a
specification instead of to a painter:

> **A group is an entity when its interface is much narrower than its internals,
> and its internals speak a vocabulary the caller never has to learn.**

`PlacedItems` is the clearest case in the pass: nine functions of occupancy
grids, cursors, dense/sparse searches and row- vs column-major scanning, behind
one call that answers *"which cell is item `i` in?"*. `GridLayout::layout` never
learns that an occupancy grid exists.

## Done

| Entity | Non-test lines | What it owns |
|---|---|---|
| `algorithm/grid_track_sizing.rs` | 770 | css-grid-2 §12.3 *Track Sizing Algorithm* — the whole staged pipeline: `Track`'s base/growth-limit pair, the `Kind`/`Contributions` vocabulary an item is measured in, §12.5 resolve-intrinsic with its span batching and its two distribute sub-steps, §12.6 maximise, §12.7's fr restart, §12.8 stretch. Plus §7.6's cycled implicit track sizes, which answer the same "what sizing function does track `n` have" question |
| `algorithm/flex_line_placement.rs` | 420 | One flex line: its flexible lengths resolved, its items' cross sizes settled, its free space handed to the `auto` margins and *then* to `justify-content` (§8.1, in that order), every item written into its box. `LineArgs` is the reason both the no-wrap and the wrap caller reach it through exactly one call shape |
| `algorithm/grid_item_placement.rs` | 351 | css-grid-2 §8.5 *Grid Item Placement Algorithm* — `PlacedItems`: the occupancy grid, the definite-lines-first two-phase pass, the sparse cursor and the dense from-the-origin search, in both the row-major and the column-major spelling, and the implicit column/row counts that fall out |
| `algorithm/resolve_flexible_lengths.rs` | 226 | css-flexbox-1 §9.7 *Resolve Flexible Lengths* — the freeze loop, the signed violations, the scaled shrink factor, the `Σfactors < 1` cap, and the two guards that exist because a layout which does not terminate takes the event loop with it |
| `algorithm/grid_track_positions.rs` | 184 | Where the tracks *sit*, once their sizes are known: §7.2.3.2's collapse of an empty `auto-fit` track, §11.5's distribution of leftover space under `justify-content`/`align-content`, the running offsets, and the extent a span reaches through them |
| `algorithm/flex_main_probe.rs` | 150 | What an item answers when it is *asked* for a main-axis size: §4.5's content-based minimum, §9.2.3's `auto` basis, and CSS Sizing 3 §4.1's three keywords — all one primitive (`probe_main`) under a different offer |
| `algorithm/grid_item_alignment.rs` | 111 | css-grid-2 §11.3/§11.4 — one `CellAlign` for both axes, and the written argument for why `stretch` is *conditional* on the item's own computed size (plus the half of that condition this engine cannot yet test, and what honouring half of it would break) |
| `algorithm/flex_axis_bounds.rs` | 105 | The item's bounds on either axis and the two clamps that apply them (§4.6 main, §9.4 step 11 cross). Pure style arithmetic: no measurer appears anywhere in the file |
| `algorithm/flex_cross_size.rs` | 103 | css-flexbox-1 §9.4 — one item's cross size from three sources (explicit, stretched, measured) through one clamp, and the percentage-basis argument that keeps the item's *offer* and its *containing block* apart |
| `algorithm/flex_main_margins.rs` | 46 | css-flexbox-1 §8.1 — what a main-axis margin edge **reserves** versus what it finally **occupies**. An `auto` edge answers differently depending on when it is asked, and calling the wrong one is how the free space gets counted twice |

Ten files, **2,466 lines of non-test code**.

## Why the placement pass took its caller's code with it

Nine of the ten extractions moved existing free functions. One did not:
`PlacedItems::resolve` is 110 lines that were living *inside* `GridLayout::layout`
— the anchor/span resolution, the implicit column and row counts, and both
placement phases — and it took them because the eight occupancy helpers below it
are otherwise a bag of functions rather than an entity. The test is the same one
the rest of the pass uses: with the phase body left behind, the caller would
still have had to know that a `Vec<Vec<bool>>` exists, that Phase A runs before
Phase B, and that the cursor is reset for a dense flow. With it moved, the caller
knows a struct with six fields.

Nothing else was lifted out of either `layout` method, and that restraint is
deliberate. Flex's line breaking (§9.3) and its `align-content` distribution
(§8.4) are the two obvious remaining candidates; both are short, both are woven
into the `place` closure that the packing pass and the measuring pass share, and
neither has internals its caller does not already have to understand. Extracting
them would be moving code to move code.

## What did not move: every single test

**No test moved, and none needed to.** Between them the two files hold 3,789
lines of `#[cfg(test)] mod tests` — 2,231 in grid, 1,558 in flex — and a grep
for every extracted item's name inside those modules finds **no call to any of
them**. The only matches are a grid test *named*
`content_distribution_axes_are_independent` (which drives `layout`, not
`content_distribution`) and four occurrences of the English word "used" in flex
comments.

That is worth stating as a measurement rather than an impression: every test in
both files drives the public `LayoutAlgorithm::layout` and asserts placed boxes.
§3.1 puts "that entity's unit tests" with the entity, and the entity these tests
exercise is `GridLayout` / `FlexLayout` — so they stay, and the two files remain
majority test (78% and 69%). Anyone reading 2,867 and 2,251 as "barely shrank"
should read 636 and 695 instead.

Adding unit tests for the ten new entities would be a real improvement and it is
**not** this pass's work: a test written now would be written against the
implementation it was just extracted from, which is how a refactor's test suite
comes to assert the refactor's own bugs. The behavioural proof this pass offers
instead is that 6,121 tests and a byte-identical `showcase_hidpi.png` all still
hold across a move of 2,466 lines.

## What this extraction found

**No new instance of the campaign's recurring doc defect.** The counter stays at
fourteen. Both files were written under heavy documentation discipline — every
one of the ten extracted entities arrived with its doc comment correctly
attached, and a mechanical check over every item in all twelve touched files
found no item left bare. That is a result about the files, and it is the first
time in five passes it has come out clean.

Three smaller things did come out:

- **The grid file cites a spec numbering that no longer exists.** Its track
  sizing is documented against §11.4–§11.8; the vendored `specs/w3c/css-grid-2.txt`
  numbers the same steps §12.4–§12.8 under §12.3, subgrid having taken §9 in the
  meantime, and a handful of the file's own citations (`§12.5`, `§12.7.1`) are
  already in the later numbering. The doc comments were moved **verbatim** rather
  than renumbered — they are correct against the draft they were written for, and
  silently rewriting a spec citation is worse than a stale one — and
  `grid_track_sizing.rs`'s module doc now states the mapping explicitly, so a
  reader with the vendored file open can follow either.
- **The sizing pipeline's *input* vocabulary is part of its interface.** `Kind`,
  `KINDS`, `Contributions` and `track_takes` had to leave `grid_layout.rs` with
  `resolve_tracks`, because the caller measures each item once per intrinsic kind
  its span crosses and hands the results in. That is the honest shape of the
  §12.5 contract — the algorithm decides *which* contributions it needs, the
  caller is the only one who can measure them — and it is why the module exports
  six names rather than one.
- **A pedantic clippy warning that predated the pass.** `place_line`'s 80-line
  `match stretch_target` is `single_match_else`; it became `if let … else` in its
  new file. The two arms are the no-wrap and the wrap semantics and both keep
  their comments; only the shape changed.

## What is left, and what it is not

Both files stay on the `file_size.rs` ledger, because that cap counts **total**
lines and the tests are 78% and 69% of what remains. Their entries no longer
claim the files are indivisible; they now say which entities left and what the
remainder is, which is what a debt register is for.

What is left in `grid_layout.rs` is the six documented steps of the formatting
context — content area, the expanded templates, the two track-sizing calls with
the measurements that feed them, the item boxes and the containing blocks they
state, and the clamped border box. What is left in `flex_layout.rs` is steps 1–2
(the container's content extents and every item's flex base and hypothetical main
size) plus the line stacking that calls `place_line` once per line. Both are
recognisably the algorithm they are named for, read top to bottom, with the
sub-algorithms named rather than inlined.

The next reduction in either file is its test module, and — as with
`layout_coordinator.rs` — spec §3.1 argues against it rather than for it.

# The sixth pass: the paragraph wrap, along the pipeline-stage seam

**Status:** done. **Eight entities extracted** from
`crates/domain/obscur-text/src/layout/paragraph_layout.rs` — **1,626 → 598 lines
of non-test code** (3,312 → 2,241 total), comfortably under the owner's
operational bar of 1,000. Suite **6,147 → 6,173 passed / 0 failed**; `fmt` clean;
`xtask check` and `xtask consumer` both pass; `clippy` **374** workspace
warnings — unchanged, and none of the eight new files carries one.

`showcase_png` renders **byte-identical**, and for once that is measured in both
directions rather than quoted: `showcase_hidpi.png` hashes `bb153594…` with the
pre-change file swapped back in and `bb153594…` again with the new one, in this
session, on this machine. The showcase wraps text on every frame, so this is a
real proof rather than a formality.

## Why the seam is different again

The `file_size.rs` entry defending this file said: *"line breaking, bidi
reordering and alignment share one line-box state"*. That is the shape the code
presented — seventeen private associated functions hanging off
`impl ParagraphLayout`, every one of them reachable from `wrap_core`, every one
of them speaking in `Tagged` glyphs. It is not what the code *was*.

There is no shared line-box **state**. `wrap_core` is a **pipeline**: flatten →
normalise advances → find breaks → split words → pack lines → re-tab → hang →
justify → truncate → segment → size. Each stage consumes the previous stage's
output and produces the next stage's input; what they share is a *vocabulary* —
a tagged glyph, a packed line — and a vocabulary is a type, not a state. Naming
the two carriers was the whole trick: once `TaggedGlyph` and `PackedLine` exist,
every stage is a free-standing algorithm with its own W3C citation and an
interface far narrower than its internals, which is the fifth pass's test for an
entity, applied to a pipeline instead of to a specification's table of contents.

This file also needed a second rule the earlier passes did not:

> **When several stages thread the same loose arguments together, those
> arguments are an entity too.**

`hyphen_offsets`, `soft` and `hang_ws` were three `HashSet<usize>` computed side
by side in `wrap_core` and then passed *as a trio* through five functions, down
to the leaf that builds a word. They are three questions about one source text —
may a word split here, did the author write a soft hyphen here, does this space
hang — and as `BreakOffsets` they answer them behind three predicates. That one
change is what took the packing step from nine arguments to seven and removed one
of the file's six `#[allow(clippy::too_many_arguments)]`.

## Done

| Entity | Non-test lines | What it owns |
|---|---|---|
| `layout/paragraph_word.rs` — `ParagraphWord` | 303 | The unbreakable unit, and every way it can still be broken: its construction from the stream at the break opportunities — including the trailing hard break past the last glyph, whose offset no glyph's cluster carries — its interior hyphenation points from the hyphenator port and from author `U+00AD`s, the widest *fitting* hyphen split, and the CSS Text 3 §5.2 emergency grapheme-cluster split that never cuts inside a cluster. `HyphenPoint` and `Split` are its satellites, not its equals |
| `layout/packed_line.rs` — `PackedLine` | 295 | The greedy fit itself, and the two corrections that only become meaningful once a line's edges are known: §4.3's tab stops re-measured from the line's **own** left edge (the pre-wrap pass could only reset at a hard `\n`), and §7.1/§8.1's trailing collapsible space and trailing tracking hung off its right edge. Both rewrite the line's advances and nothing else, which is the invariant that keeps `width()` honest |
| `layout/break_offsets.rs` — `BreakOffsets` | 125 | The three byte-offset tables the wrap asks about the source text, collected once from one `text` and one **finalized** break list — the ordering matters, because `word-break: keep-all` removes breaks and the hyphenation offsets are collected *between* them |
| `layout/line_segments.rs` | 106 | One line's logical glyphs cut into style-homogeneous `ShapedRun`s (a boundary in any of font, level, size, faux-bold, faux-oblique) and reordered by the UAX#9 L2 rule. The reorder is a pure function of the levels, which is why it can now be tested as one |
| `layout/line_strut.rs` — `LineStrut` | 95 | CSS 2.1 §10.8.1: the strut **and** the line-box formula it defines — the `max` against the fonts actually on the line, then half-leading above and below. This was already the file's second **public** entity, exported beside `ParagraphLayout` from one `pub use`, which is §3.16's "one file must not export several main entities" in its plainest possible form |
| `layout/line_truncation.rs` | 87 | `text-overflow`: the largest prefix that fits, the ellipsis advance reserved before measuring it, and the carrier glyph the ellipsis inherits its font, direction, level and cluster from so it segments and reorders with the text it truncates |
| `layout/tagged_glyph.rs` — `TaggedGlyph` | 70 | The vocabulary every stage speaks: one glyph plus the provenance (font, direction, size, bidi level, synthesis) that lets segments be rebuilt after packing has forgotten which run a glyph came from. It also answers the one question the whole pipeline keeps asking of the source text — *what character did this glyph come from* |
| `layout/line_justification.rs` | 65 | `text-align: justify`: which lines are eligible (not the last, not hard-broken, at least two words) and how the surplus is spread across their gaps. Eligibility is a paragraph-level fact, which is why it is not a `PackedLine` method |

Eight files, **1,146 lines of non-test code**.

## Why justification and truncation are files and not methods

Both are one pass over `&mut [PackedLine]`, and both could have been folded into
`packed_line.rs` — or onto `TextAlignment` / `TextOverflow`, where the CSS value
they implement already lives. Neither was, for the same reason twice.

`PackedLine` owns what is intrinsic to a line's *measurement*: the tab and hang
corrections exist because the line has edges. Justification and truncation are
the application of a **declared property**, each with its own eligibility rule
and its own vocabulary (gaps and surplus; carrier and reserved advance) — a
different question from "how wide is this line".

And `text_alignment.rs` / `text_overflow.rs` are the value files the prescribed
§3 layout names; `text_alignment.rs` is used by the *single-font*
`TextLayout` path as well, so putting the paragraph's justification pass there
would drag `PackedLine` and `TaggedGlyph` into a file that has no business
knowing they exist.

## What did not move: the tests, with one exception

1,643 of the file's remaining 2,241 lines are its `#[cfg(test)] mod tests`, and
they stay. A grep through them finds no call to any extracted item: every test
drives one of the four public `wrap*` entry points and asserts laid-out lines,
so the entity they exercise is `ParagraphLayout` and §3.1 keeps them with it.

**Exactly one test moved.** `segment_splits_on_font_size_and_synthesis_boundaries`
builds tagged glyphs by hand and calls the private `segment` directly, so its
subject moved to `line_segments.rs` and it moved with it — same assertions, same
comment, two renames.

**Twenty-six tests were added** (27 in the new files, minus the one that moved),
and the fifth pass's warning about this is worth answering rather than ignoring:
a test written immediately after an extraction can end up asserting the
refactor's own bugs. These do not, because each one asserts a rule that a doc
comment **predating this pass** already states — the strut is a floor and not a
cap, an emergency cut lands only on a cluster boundary, a hyphenation offset at a
word edge is dropped, L2 reverses a maximal `>= level` span, a hung space keeps
its cluster so hit-testing still sees it. The behaviour itself is pinned
independently by the 48 wrap-level tests that did not move and by a
byte-identical showcase.

## What this extraction found

**No new instance of the campaign's recurring doc defect.** The counter stays at
fourteen, and this time it is a measurement rather than an impression: a
mechanical pass over every item in the pre-change file found that the only ones
without a doc comment directly above them are `impl` blocks and private struct
fields whose struct doc covers them collectively — the same style the file used
throughout. Nothing was left bare by a missing blank line.

Three other things did come out:

- **An argument list that was an entity.** The `hyphen_offsets` / `soft` /
  `hang_ws` trio, described above. It also absorbed the ~40 lines of collection
  code that were sitting inline in `wrap_core`, which is why that function now
  reads as a list of named stages.
- **Two helpers that had outlived their shape.** `char_at(text, cluster)` and
  `is_word_separator(text, cluster)` both took a byte offset that every caller
  obtained from a glyph it already had in hand. As `TaggedGlyph::source_char` and
  `TaggedGlyph::is_word_separator` they take no offset at all, and the question
  they answer — *what character is this glyph?* — belongs to the glyph. That is
  what let both of them stop being free functions shared across the module.
- **A stale cross-crate doc reference.** `shaping/spacing.rs` names the stage
  that performs §8.1's trailing-tracking trim, and named it
  `ParagraphLayout::wrap_core` → `hang_line_trailing`. The stage is now
  `PackedLine::hang_trailing`, and the module doc says so.

## What is left, and what it is not

The 598 lines are: `ParagraphLayout` itself (four fields, `new`, `aligned_to`,
four accessors), the four public `wrap*` entry points — whose ~250 lines are
almost entirely the doc comments a caller reads, and which rule 3 of this
campaign keeps exactly where they are — `wrap_core`, now a sequence of named
stages with the reasoning for each kept at the call site, and
`is_keep_all_suppressed`, the sixteen lines where the wrap *edits* the break list
before anything else consults it.

The file stays on the `file_size.rs` ledger, because that cap counts **total**
lines and 73% of what remains is its test module. Its entry no longer claims the
file is one entity sharing one state; it now says which eight entities left and
what the remainder is.

---

# The seventh pass: the cascade's tables, along the "one fact per property" seam

**Status:** done. **Nine entities extracted** from the two remaining oversized
files in `obscur-style`, and both are now under the owner's operational bar of
1,000 lines of non-test code:

* `computed/style_resolver.rs` — **1,167 → 429 lines of non-test code**
  (2,338 → 1,322 total);
* `property/property_id.rs` — **1,084 → 725 lines of non-test code**
  (1,550 → 1,191 total).

Suite **6,173 passed / 0 failed** — unchanged, with no test weakened, rewritten
or deleted; three tests **moved** with the subject they drive. `clippy` **374 →
372**; `fmt` clean; `xtask check` and `xtask consumer` both pass. `cargo doc`
reports the same seven warnings against `obscur-style` it reported before, none
of them against a file this pass created.

## Why the seam is different again

Neither file was a god object and neither was one algorithm. Each was **one
entity plus a set of tables about it**, and §3.16's question — how many main
entities does this file export? — had an embarrassingly literal answer in both:

| File | Second export |
|---|---|
| `computed/style_resolver.rs` | `pub use style_resolver::{ResolutionContext, StyleResolver};` |
| `property/property_id.rs` | `pub use property_id::{PropertyId, WritingMode};` |

Both `mod.rs` lines said the violation out loud. `ResolutionContext` is not a
detail of the resolver, it is the resolver's *input* — a value the caller builds
once per document and hands to every node — and `WritingMode` is not a member of
the property catalogue, it is the computed **value** of one of the properties in
it. Neither had ever needed to be in the file it was in.

Past those two, the seam this pass follows is the prompt's own rule:

> **A table that maps every property to one fact is a unit; several such tables
> in one file are several units.**

`style_resolver.rs` held four of them (the property set it walks, the initial
value of every property, the choice between a logical declaration and its
physical counterpart, and the relative-`font-weight` bucket table) plus one
sub-algorithm (the effective custom-property set and the `var()` substitution
that reads it). `property_id.rs` held two derived classifications (the
logical→physical mapping and the paint-only allow-list) that the catalogue does
not state variant by variant.

## Done

| Entity | Non-test lines | What it owns |
|---|---|---|
| `computed/initial_value.rs` — `initial_value_of` | 330 | What every property computes to when nothing declares it, and *which* ones this phase must leave unresolved because their initial is an interned keyword. One fact per `PropertyId`, asked from three unrelated places in the pass (no cascaded value, an explicit `initial`, an `unset` on a non-inherited property). It also owns `INITIAL_FONT_SIZE`, which is that table's `font-size` row read by name |
| `computed/computed_property_set.rs` | 163 | The two lists that say what one pass covers: every longhand the resolver computes, and the face-selecting subset it must resolve early so the `ch` port has a face to measure in. Kept exhaustive by a 257-line test that has nothing to say about how a value is computed |
| `computed/resolution_context.rs` — `ResolutionContext` | 125 | A **second public entity** that was living in the resolver's file. The three length bases one node's cascade cannot supply — the root's `font-size`, the viewport, and the port that measures the `0` glyph — plus the host `env()` values |
| `computed/logical_cascade.rs` | 120 | Spec §4.13: `margin-inline-start` and `margin-left` are separate `PropertyId`s that cascade independently, so the winner between them is not a cascade result — it is decided here, by the priorities the cascade recorded. The two axis keywords (`resolves_rtl`, `resolves_writing_mode`) are in the same file because they exist **only** to feed `PropertyId::physical_under`, and the resolver reads them before any box-model property precisely so this answer cannot depend on iteration order |
| `computed/relative_font_weight.rs` — `resolve_relative_weight` | 82 | §4.14's nine-bucket `bolder` / `lighter` table, the rounding into buckets, and the parent reading it is keyed on. Its interface narrowed in the move: three items the resolver called in sequence became one question it asks |
| `computed/effective_custom_properties.rs` | 71 | Steps 1–2 of the pass: the node's effective custom-property set (the parent's, overridden by its own — custom properties always inherit) and the `var()` substitution that reads it, with the visited-set guard that makes a cycle terminate as `unset` |
| `property/logical_property.rs` | 200 | The other half of §4.13, on the catalogue's side: which physical longhand each of the twenty logical ones names, under a given `direction` and `writing-mode` (css-writing-modes-4 §6.2), plus the membership test and the list to scan |
| `property/paint_only.rs` | 87 | Whether a change to a property can change a box (§3.4 phase 5, §4.3) — the conservative allow-list the frame's invalidation classifier consults, and the `affects_layout` inverse. It arrived with a test module of its own, *beside* the catalogue's name/inheritance tests, which is the seam showing before it was cut |
| `property/writing_mode.rs` — `WritingMode` | 46 | The other **second public entity**. `obscur-layout` already had a `model/writing_mode.rs`; this is the style side of the same value |

Nine files, **1,224 lines of non-test code**.

## What did not move, and why

- **`PropertyId::name` and `PropertyId::default_inheritance`.** They are two more
  tables over the same catalogue — 128 and 142 lines — and moving them would have
  taken `property_id.rs` to about 460. They stay, and the reason is not their
  size: **every variant's own doc comment already states both facts** ("The
  `background-color` property", "(inherited)" / "NOT inherited"), so the tables
  are the machine-readable half of the documentation sitting a few hundred lines
  above them. The file's ~30 unit tests agree — each asserts a property group's
  name *and* its inheritance in one test (`background_size_position_repeat_names_and_not_inherited`),
  so splitting the tables would have split every one of those tests down the
  middle. What left instead is the two facts the catalogue does **not** state
  variant by variant.
- **`resolve_font_size` and `font_context_of`** (89 lines). The one phase of the
  resolver that is genuinely its own — `font-size` is resolved before every other
  property because it is the base its own node's `em` resolves against — but it
  calls the resolver's own `resolve_property`, so extracting it would have had to
  widen that private method into a `pub(super)` one to let another file call back
  in. It is the pass's first step, and it stays in the pass.
- **The `logical` test module** (167 lines, seven tests). Its subject is the
  §4.13 cascade choice that moved to `logical_cascade.rs`, but every one of its
  tests drives `StyleResolver::resolve` and asserts a computed `margin-left`, so
  §3.1 keeps them with the entity they exercise. **Six tests moved**, in three
  groups, and each moved because it calls a moved item **directly**:
  `computed_properties_cover_every_property_id`, `relative_font_weight_table`,
  and the four tests of the `paint_only_tests` module.
- **The public API.** `ResolutionContext` and `WritingMode` are re-exported from
  their new homes by the same `pub use` lines their crates' `mod.rs` already
  carried, so `obscur_style::computed::ResolutionContext` and
  `obscur_style::property::WritingMode` resolve exactly as before —
  `style_coordinator.rs` imports the first by that path and was not touched.
  `StyleResolver::initial_value` keeps its signature and its doc comment and
  delegates; `PropertyId::physical_under`, `is_logical`, `LOGICAL`,
  `is_paint_only` and `affects_layout` keep theirs by moving as further
  `impl PropertyId` blocks, which rustdoc renders on the same type page.

## What this extraction found

**No new instance of the campaign's recurring doc defect.** The counter stays at
fourteen, and it is a measurement rather than an impression: a mechanical pass
over every item in all eleven touched files finds none left bare by a missing
blank line.

Four other things did come out:

- **Two clippy warnings, both in the code that moved, both accounted for.**
  `cascaded_for` carried a `needless_lifetimes` (`'a` on the only reference in
  its signature); it is elided in its new home rather than carried into it. The
  second is subtler and is a *silencing*, not a fix, so it is recorded here
  rather than claimed: `StyleResolver::initial_value` carried a
  `missing_panics_doc` because four of its arms `expect()` on a conversion that
  cannot fail. The `expect`s are now inside the `pub(super) initial_value_of`,
  which the lint does not apply to. Nothing about the code's ability to panic
  changed; adding a `# Panics` section to a function that cannot panic would have
  been worse than either.
- **The initial `font-size` was spelled twice, sixty lines apart.** The resolver's
  `INITIAL_FONT_SIZE` const and the `PropertyId::FontSize` arm's literal `16.0`
  were the same fact in two places; the extraction put them in the same file, and
  the arm now reads `Length::px(INITIAL_FONT_SIZE)`. Identical value, identical
  type — the only change of expression this pass made.
- **`ResolutionContext` was documented as if it were the resolver's.** Its own
  doc comment opens "The document-wide bases a computed `<length>` is absolutized
  against", which is a sentence about the resolver's step 4 rather than about the
  type; the type's module doc now says what the type is for (an input built once
  per document, handed to every node) and the original text is untouched below it.
- **`is_paint_only`'s test module was already named differently.** The file had
  `mod paint_only_tests` beside `mod tests` — a file with two test modules is a
  file with two subjects, and it named which one was the passenger. It is `mod
  tests` in `paint_only.rs` now, with all four tests and their assertions
  unchanged.

## The ledger

`crates/domain/obscur-style/src/computed/style_resolver.rs` is **removed** from
`tools/xtask/src/rule/file_size.rs`. Its entry read "the cascade is one pass over
one property set" — which was true, and was not a reason for the file to hold the
property set, the initial values, the logical mapping, the weight table and a
second public entity as well. At 1,322 total lines it is under the 2,000-line cap
and the rule fails on a stale entry, so the entry had to go rather than be
rewritten. `property_id.rs` was never on the ledger.

Both files are now under the operational bar on the measure this campaign is held
to — 429 and 725 lines of non-test code — and neither needed an argued shortfall.

---

# The eighth pass: the load job, along the thread boundary

**Status:** done. **One entity extracted**, and
`runtime/resource_bridge.rs` went from **1,077 to 984 lines of non-test code**
(1,130 → 1,037 total). Suite **6,178 passed / 0 failed** (from 6,173 — the five
are the new file's own unit tests, and no existing test was weakened, rewritten
or deleted); `clippy` **372 → 372**; `fmt` clean; `xtask check` and `xtask
consumer` both pass. Under the owner's operational bar of 1,000 lines of
non-test code this file is **done**, with no argued shortfall.

This is the first target the campaign wrote itself. `resource_bridge.rs` was the
**first** extraction of the first pass, and it landed at 1,129 lines — 77 over a
bar that did not exist yet. So the interesting question is not "how did this file
get big" but "what did the first pass, asking about *fields*, have no reason to
look at".

## The seam is a thread boundary

The bridge's fourteen fields do share the invariant the first pass extracted them
for, and nothing here disturbs that. What sat in the file *besides* them was a
**load job**: a value that is handed to a
[`BlockingExecutor`](obscur_task::BlockingExecutor) and run on another thread, or
parked in a queue and run on the frame thread after the frame phase (spec §3.3,
§3.4) — and which on neither path may reach back into the runtime.

That constraint is what makes it an entity rather than a method, and the test is
mechanical rather than aesthetic: **everything the job needs must already be in
its hands when it starts**, so its interface is exactly its arguments — the two
ports, the §5.16 candidate ladder, and the handle and version that say what it is
answering for. Its internals (the DPI ladder, the intrinsic size the decoder
reports, the RGBA byte-length guard that catches a decoder disagreeing with
itself) are reachable from nothing else in the crate. Held in `ResourceBridge`'s
file they looked like private helpers of a struct **none of them can touch**.

| Entity | Non-test lines | What it owns |
|---|---|---|
| `runtime/resource_outcome.rs` — `ResourceOutcome` | 168 | What one finished resource-load task produced, and the load that produces it: the densest DPI variant that resolves, decoded and tagged with its density; the vector counterpart; and the handle/version the hub rules staleness on |

The two constructors are new as *names* only — `load_image` and `load_vector`
each absorbed the struct literal their caller was writing by hand, so the
bridge's two spawn methods now read `ResourceOutcome::load_image(&*loader,
&*decoder, &candidates, key, handle, version)` and the `Send` closure around it
is two lines instead of eight. No behaviour changed and no signature on
`ResourceBridge` moved.

## What did not move, and why

- **The deferred queue.** It looks like part of the load path and is not: a
  drained job's outcome goes to `apply_outcome`, which asks the hub about
  staleness and seats pixels in the cache through its §5.19 budget. That is three
  of the fourteen fields, so the queue and its drain stay with the state they
  commit into.
- **The resolve passes.** They decide *what* to ask for by walking the computed
  styles and the retained tree, and they mark each URL pending in `image_handles`
  at enqueue time — the dedupe that stops a load re-queuing. Frame-thread work
  over frame-thread state, on the other side of the same boundary.
- **The diagnostics half.** `report_unresolvable_source` reads and writes the
  dedupe set; `report_failure` reads `image_referrers` to attribute the fault to
  the node that asked (§1.6). Both are the bridge's, and
  `localized_diagnostic_message` — the §7.2 code→language catalog under the
  second — is the one item here with a claim to independent semantics. It stays:
  it is thirty lines, its only caller is `report_failure`, and its one entry's
  arguments are a `ResourceError`. Moving it would have been a split made to
  reach a number the file already reaches without it.

## What the prize looks like

Five unit tests now drive the load path with **no bridge, no runtime and no real
ports** — two stubs, forty lines. Four of them could not be written before,
because the only route into `load_and_decode_image` was to install providers on a
whole runtime and render a frame: that the densest variant that resolves wins and
carries its density (so an `@2x` pixel lays out at half a logical pixel), that a
missing variant falls through to the authored path, that an unresolvable ladder
reports the **authored path's** error rather than naming an `@3x` file the author
never wrote, and that a decoder claiming a size its buffer does not match is a
decode failure rather than a panic.

## What this extraction found

**No new instance of the campaign's recurring doc defect.** The counter stays at
fourteen. It is a measurement, not an impression: a mechanical pass over every
item in both files — struct, fields, every method, every free function, both
enum variants — finds none left bare by a missing blank line. The two the first
pass recorded here (#1 `size_replaced_elements`, #2 `decode_one_image`) are still
correctly attached, and `decode_one_image`'s doc travelled with it.

The one `clippy` warning either file carries is `manual_let_else` on
`is_vector_source`, which predates this pass, sits in code this pass did not
touch, and is left alone rather than swept up in a refactor.

## The ledger

`resource_bridge.rs` was never on `tools/xtask/src/rule/file_size.rs`: that cap
counts **total** lines and 1,037 is well under 2,000. It is the operational bar
of 1,000 non-test lines that this file was over, and it is under it now.

---

# The ninth pass: the Vello adapter, along the "what the GPU submits" seam

**Status:** done. **Five entities extracted** from two files, and both are under
the bar with no argued shortfall:

| File | Non-test lines | Total |
|---|---|---|
| `conversion/scene_converter.rs` | 1,037 → **639** | 1,633 → 1,238 |
| `renderer/vello_renderer.rs` | 1,052 → **694** | 1,304 → 946 |

Suite **6,178 passed / 0 failed** — unchanged, with no test weakened, rewritten
or deleted and none added; `clippy` **372 → 366**; `fmt` clean; `xtask check` and
`xtask consumer` both pass. The six warnings are not incidental — four of them are
the direct consequence of the misplaced-doc defect this pass found, described
below.

## Why the seam is different again

This is the first adapter the campaign has cut, and an adapter's file does not
grow the way a coordinator's does. There is no god object here: `SceneConverter`
has four fields and `VelloRenderer` has ten, and every one of them is read by the
thing the type is named for. So neither rule 1 (fields sharing an invariant) nor
the pass-shaped seams of the fourth and sixth passes had anything to bite on.

What the two files held instead was decided by **what the GPU has to be told, and
when**:

- In the converter, most commands are a *mapping* — a path becomes a `BezPath`, a
  clip becomes a bounded `push_layer`, a blend becomes a `peniko::Mix`. Two are
  not. A `DrawShadow` and a `DrawImage` are **derivations**: eight radii eased by
  §6.1.1 and collapsed to the one scalar Vello's blur takes, a `DestOut` punch for
  the inset form; a tile index range per axis, a bounded-loop guard, a region
  clip, a zero-copy `Arc<[u8]>` → `Blob` wrap. Each is a paragraph of CSS with a
  one-line interface, which is the third pass's rule verbatim.
- In the renderer, the seam is a **submission boundary**. Vello rasterizes with
  its own `render_to_texture` submits; the adapter's owned wgpu passes may only
  read those textures afterwards. Every awkward shape in that file — a vector of
  prepared offscreens carried between two loops, the same thirty-line preparation
  written out twice, a scratch texture kept alive until an encoder finishes — is
  that boundary showing through.

## Done

| Entity | Non-test lines | What it owns |
|---|---|---|
| `conversion/shadow_painter.rs` | 202 | A `box-shadow` as Vello sees it: the §6.1.1 spread easing, the CSS-radius → gaussian-σ mapping, the lossy collapse to one corner scalar (conformance-audit 4-A), and the inset form's clip → fill → `DestOut` punch |
| `renderer/segmented_compositor.rs` — `SegmentedCompositor` | 220 | The ordered content/backdrop state machine a `backdrop-filter` forces. It frosts *the frame accumulated so far*, which is a property of the whole list's order rather than of any command, and so is the one effect that cannot live inside a single Vello scene |
| `renderer/filter_layer_pass.rs` — `PreparedFilterLayers` | 216 | The `filter: blur()` / colour-op sub-scenes between their two GPU steps: rendered into ~3σ margin-padded offscreens by Vello, then colour-matrix → blur → composite through the owned wgpu passes. **Both** render paths drive it, which is what keeps a filtered element identical whether or not the same frame also carries a backdrop |
| `conversion/image_painter.rs` | 178 | A `DrawImage` as a CSS background *layer*: the per-axis tile index range, the `MAX_TILES_PER_AXIS` / `MIN_TILE_EXTENT` guards, the region clip, and the `SharedPixels` newtype that makes the `Arc<[u8]>` → peniko `Blob` wrap a refcount bump rather than a pixel copy |
| `conversion/scene_segment.rs` — `SceneSegment` | 64 | A second **exported** entity that was living in the converter's file — §3.16 in its plainest form, as `ScrollPaint` was in the third pass |

Five files, **880 lines**. What is left in `scene_converter.rs` is the command
walk itself, the four public entry points, the font registry, and the small
helpers the walk alone uses (the layer-effect lookups, the bracket matcher, the
transform maths, the blend/composition tables). What is left in
`vello_renderer.rs` is the port implementation, the three entry points a host
calls, and the device/blitter/offscreen caching that is the renderer's own state.

## What did not move, and why

- **`mix` and `compose`.** Two tables mapping the domain's `BlendMode` /
  `CompositionMode` onto peniko's. They are the seventh pass's "one fact per
  property" shape at a twelfth of the size, and their only caller is `push_layer`
  four lines away. Twenty-eight lines behind a file boundary would be a split made
  to reach a number the file already reaches without it.
- **The main-scene fast path.** `render_to_frame` converts, rasterizes, and
  dispatches to the two extracted passes. That sequence *is* what the type is for.
- **The shadow and image tests.** They drive `SceneConverter::convert` and assert
  the resulting encoding, so their subject is the converter's public API, not the
  painter's internals — the third pass's rule for which tests travel. They stay
  where they are, and the two painters ship with none of their own; nothing in the
  suite was lost or weakened to arrange that.
- **The `collapsible_if` in `cached_offscreen`.** Pre-existing, in code this pass
  did not touch.

## What this extraction found

**One instance of the campaign's recurring doc defect — the fifteenth — and it is
the worst-behaved variant seen so far**, because the misplaced block dragged an
attribute with it:

| Found by | The item left bare | Where its doc had gone |
|---|---|---|
| `PreparedFilterLayers` | `render_to_frame` | `cached_offscreen`, seventeen lines and an `#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]` above the two-line doc that item actually has |

So the renderer's shared core — the one method the port `render`, the on-screen
present and the headless read-back all funnel through — carried **no
documentation**, while a two-line cache helper carried a `# Errors` section for
errors it cannot return. And because the `#[allow]` travelled with the text, the
casts it was written for (`(blur.bounds.size.width + 2.0 * margin).ceil() as u32`
and its neighbour) were **unsuppressed**: live `clippy` warnings in a file whose
author believed he had allowed them. That is the *entire* 372 → 366 difference —
two cast sites × two lints, counted across the lib and lib-test targets — and it
is measured, not inferred: `casting f32 to u32` had six hits in this crate before
the pass and has none after.

Both halves are restored verbatim: the doc and the attribute sit on
`render_to_frame`, and `cached_offscreen` keeps its own two lines.

Two smaller repairs, both in doc text that was already broken where it stood and
was fixed rather than carried: `[`CornerRadius::circular_approximation`]` and
`[`Compose::DestOut`]` had no resolvable target in the converter's file (neither
type is imported there) and now name theirs explicitly, so the two paragraphs that
explain the shadow backend's *documented* losses actually link to the functions
they blame.

## The proof

The adapter's unit tests deliberately run without a GPU, so nothing in the suite
executes the wgpu code this pass moved. The proof is therefore a render
comparison, and it is an A/B rather than a citation: a worktree carrying the
current tree with **only** the Vello adapter reverted was built and run alongside
this one, and all six headless Vello examples came out **byte-identical**.

| Example | md5 | What it proves moved intact |
|---|---|---|
| `uikit_vello_png` (skeuomorphic) | `9f88b478…` | `shadow_painter` — the theme's `box-shadow` carries an `inset` gloss **and** an outset drop shadow in one list, so both halves of the file run |
| `multi_bg_proof` | `47224af9…` | `image_painter` — multiple background layers, tiled |
| `backdrop_proof` | `19166649…` | `SegmentedCompositor`, end to end |
| `color_filter_proof` | `220cf2ca…` | `PreparedFilterLayers` with a colour matrix (the scratch-texture path) |
| `blur_proof` | `9f936a94…` | `PreparedFilterLayers` with a blur (the margin/offset maths) |
| `capabilities_vello_png` | `0544d85c…` | the whole page through the fast path |

## The ledger

Neither file was ever on `tools/xtask/src/rule/file_size.rs` — that cap counts
**total** lines, and 1,633 and 1,304 are under 2,000. It was the operational bar
of 1,000 non-test lines that both were over, and both are under it now.

---

# The tenth pass: the CSS parser, along the lexical boundary

**Status:** done. **Two entities extracted**, and
`parser/stylesheet_parser.rs` went from **1,039 to 607 lines of non-test code**
(1,304 → 732 total). Suite **6,182 passed / 0 failed** (from 6,178 — the four are
the new cursor file's own unit tests, and no existing test was weakened,
rewritten or deleted); `clippy` **366 → 366**; `fmt` clean; `xtask check` and
`xtask consumer` both pass. Under the owner's operational bar of 1,000 lines of
non-test code this file is **done**, with no argued shortfall.

## The seam is announced by the file's own imports

This is the plainest §3.16 case the campaign has met, and the evidence is not a
line count: **five sibling files were importing from `stylesheet_parser`**.

```rust
use super::stylesheet_parser::{CssToken, CssTokenKind, TokenCursor, Tokenizer, slice_span};
```

`selector_parser`, `property_parser`, `value_parser`, `media_query_parser` and
`container_query_parser` each opened with a line of that shape. None of them
parses a stylesheet, and none of them calls `StylesheetParser` — they were
reaching *through* the top-level parser's file to get at a lexical layer that
merely happened to live in it. An import list is the cheapest possible measure of
"whose entity is this", and this one had been answering "not the stylesheet
parser's" in five places.

The file's own module doc said so too: it opened "**The CSS tokenizer and** the
top-level `StylesheetParser` entry point" — one file, two entities, named in the
first line.

## Done

| Entity | Non-test lines | Total | What it owns |
|---|---|---|---|
| `parser/css_tokenizer.rs` — `Tokenizer` | 377 | 522 | The scanner and its output alphabet: `char_indices` walking (so a multibyte character is never split), byte-accurate spans, the ident/number/dimension start predicates, and the *totality* guarantee — an unterminated string or comment is a recorded `CssParseError` beside a best-effort token stream, never a panic and never an abort |
| `parser/token_cursor.rs` — `TokenCursor` | 92 | 140 | Reading an already-tokenized slice: whitespace-aware navigation over a borrowed `&[CssToken]` (whitespace stays in the stream because a selector needs it as a descendant combinator), plus the end-of-input convention that `here_span` and `slice_span` must agree on |

Two files, **469 lines**. What is left in `stylesheet_parser.rs` is the top-level
rule reader — the `parse_rules` loop, the at-rule dispatch (`@media`,
`@container`, `@keyframes`, and the "recognised but unsupported" report for the
rest), the declaration-block reader, the four recovery walks, and the rule AST
those produce.

The interface test the third pass set is met by both with room to spare. The
tokenizer is **one call** — `Tokenizer::new(text).tokenize()`, text in,
`(tokens, errors)` out — over 250 lines of scanning rules that nothing else in
the crate can reach or wants to. The cursor is six one-line methods over a
borrowed slice.

## Why the token model went with the scanner and the cursor did not

`CssToken` and `CssTokenKind` are the tokenizer's **output alphabet**: they have
no meaning apart from what it emits, `CssToken` has no behaviour beyond three
accessors, and every variant's documentation is a statement about what the
scanner strips (`Hash` drops its `#`, `Str` drops its quotes, `AtKeyword` drops
its `@`). Keeping them with it is also the shape this crate already uses
everywhere else: `selector_parser.rs` holds `SelectorParser` *and* the six parsed
selector types, `property_parser.rs` holds `PropertyParser` *and*
`ParsedDeclaration`. A producer and the AST it produces is one entity here, and
this pass leaves `stylesheet_parser.rs` in exactly that shape too — the parser
plus the eight rule types it returns.

`TokenCursor` is not that. It is the **reader** half, it holds no text and
allocates nothing, and the decisive fact is negative: **`StylesheetParser` does
not use it at all.** The top-level `Builder` walks its own owned `Vec` with its
own `cur_kind`/`cur_span`/`skip_ws`, which is why the cursor's methods and the
builder's near-identical ones could drift apart unnoticed for as long as they
shared a file. Its callers are the sub-parsers, all of which take a *slice* of a
stream someone else tokenized. Filed with the scanner it would have been the one
item in that file with no relationship to the scanner's state.

`slice_span` goes with the cursor rather than with the token model because of the
convention it carries: an absent token is reported as an **empty span at the
recorded `eof`**, not as no span at all, and `TokenCursor::here_span` and
`slice_span` have to keep answering that the same way or a diagnostic points at a
different place depending on which route raised it. That is now written down on
the file rather than implied by adjacency.

## What did not move, and why

- **The recovery walks.** `recover`, `skip_at_rule` and `skip_block_body` read as
  a self-contained "resynchronize" sub-algorithm, and they are not one: all three
  are `Builder` methods over the builder's own position, they call each other,
  and their vocabulary (a rule boundary, a balanced block) is the rule reader's.
  §3.1 admits exactly this — private local functions with no independent
  semantics stay with their entity.
- **`find_lbrace` and `collect_declaration`.** Both are the same shape: a
  depth-counting scan the rule reader alone performs, over the rule reader's own
  token vector.
- **The declaration reader.** The obvious second seam here would be "the block's
  declaration loop", but it is 40 lines that delegate to `PropertyParser` — the
  declaration reader is *already* a separate file, and what stayed is the loop
  that finds each declaration's token range for it.
- **The rule AST.** Eight `pub` types, and every one of them is what
  `StylesheetParser::parse` returns; see above.

## What this extraction found

**No new instance of the campaign's recurring doc defect. The counter stays at
fifteen.** As in the eighth pass this is a measurement rather than an impression:
a mechanical pass over every item in all three files — every struct and its
fields, every enum and its variants, every method and every free function — finds
none left bare by a missing blank line, and nothing whose documentation describes
its neighbour. Every doc comment moved verbatim with its item.

Two module docs are new text rather than moved text, and they are the only prose
this pass wrote: each new file says what its entity is and why the other one is
not it, and `stylesheet_parser.rs`'s opening line no longer claims to own a
tokenizer it no longer contains.

## The proof

The suite, and specifically the 457 unit tests in `obscur-css` — every one of
which reaches the tokenizer, because there is no other way into this crate. The
twelve tokenizer tests travelled to `css_tokenizer.rs` unchanged (identifiers and
custom properties, at-keywords and hashes, numbers/percentages/dimensions,
`.5` versus `.foo`, `1e3` versus `1em`, both quote forms, the two unterminated
recoveries, punctuation, byte-accurate spans, multibyte safety); the ten
stylesheet tests stayed. The four new ones are `TokenCursor`'s own and could be
written for the first time only in the sense that nobody had: that `advance`
stops at the end rather than running past it, that whitespace is in the stream
until `skip_ws` is asked, that `here_span` collapses to an empty span at `eof`,
and that `slice_span` covers first-through-last and falls back to `eof` when
handed nothing.

## The ledger

`stylesheet_parser.rs` was never on `tools/xtask/src/rule/file_size.rs` — that
cap counts **total** lines and 1,304 is under 2,000. It was the operational bar
of 1,000 non-test lines that it was over, by 39, and it is 393 under it now.

# The passes that closed without a chapter

Three stages of the second campaign landed their code and their `file_size.rs`
ledger edits but never wrote themselves up. They are recorded here, from their
own reports, with every line count re-measured on the tree rather than copied.

## The CSS property compiler, along the fan-in seam

`crates/frontend/obscur-css/src/compiler/property_compiler.rs`, **4,467 → 514**
non-test lines.

The `file_size.rs` ledger claimed "one `match` arm per CSS longhand over 119
properties; the arms are the entity", and study did not survive contact with the
file: `lower_for_property` is **212 of 4,467 lines**. The seam is not property
family but **helper fan-in**. `dimension_length` is read by six families,
`parse_stop_color` / `named_color` / `parse_hex_color` by four, `angle_to_degrees`
by three — those are the crate's floor. Everything reachable from exactly *one*
longhand family has an interface far narrower than its internals, which is what
makes a family a file rather than a section.

Thirteen entities left, each beside its parent in `compiler/`:
`gradient_lowering.rs` (647), `shorthand_expansion.rs` (593),
`background_lowering.rs` (430), `border_shorthand.rs` (409),
`color_lowering.rs` (403), `longhand_catalog.rs` (314),
`filter_lowering.rs` (295), `grid_track_lowering.rs` (295),
`value_lowering.rs` (272), `transform_lowering.rs` (246),
`grid_placement.rs` (112), `shadow_lowering.rs` (107), `font_lowering.rs` (96).
`property_compiler.rs` keeps the public `PropertyCompiler` /
`CompiledDeclaration` contract, `compile` / `compile_all`, the dispatch and
`narrow_f32`.

All 211 unit tests moved with their subjects (203 relocated, 8 kept on the entry
point); no assertion changed and `obscur-css`'s test count was identical across
the move. One cross-family dependency could not be removed without moving a
parser away from the property it is named for — the gradient `at <position>`
parser borrows `transform-origin`'s `origin_component` / `OriginAxis` — so it is
stated in `transform_lowering.rs`'s module doc instead of being invisible.
`box_side_length` was referenced outside its family only from a *comment*; it is
private in its new home rather than exported to satisfy a mention.

Two statements in the module doc had gone false and were corrected: "one of the
seven `PropertyId` longhands" (the table holds 118) and "multi-component
shorthands are beyond the domain's value model, so lowering is best-effort on the
first component" (untrue of every shorthand, gradient, track list and shadow in
the file). No instance of the campaign's recurring doc-misattachment defect
exists in this file; every item/doc boundary was checked mechanically.

## The example scene, along the "each stylesheet is a design system" seam

`crates/facade/obscur-kit/examples/uikit/scene.rs`, **2,725 → 95** non-test lines.

The file exported eleven main entities, not one: nine complete per-theme
stylesheets, the theme table that indexes them, and the authored view tree. Each
stylesheet is a whole design system authored against the shared semantic class
vocabulary whose entire interface is one `&'static str`, so each got its own
`<name>_theme.rs`: `skeuomorphic_theme.rs` (355), `fluent_theme.rs` (276),
`dark_saas_theme.rs` (258), `minimal_theme.rs` (255), `lara_theme.rs` (251),
`metro_theme.rs` (250), `material_theme.rs` (247), `flat_theme.rs` (247),
`neutral_theme.rs` (44). The `(name, css)` table and its index vocabulary became
`theme_catalog.rs` (54) — the one item that knows every stylesheet — and the view
split into `component_gallery.rs` (440), `theme_switcher.rs` (56, with the
`KIT_TAB_CMD` bindings it owns) and `classed_node.rs` (25, the `el`/`text`
vocabulary the whole scene is authored in). `scene.rs` is left as the composer.

Equivalence was proved mechanically rather than by a PNG baseline, because no
baseline existed: a script extracted all 41 item bodies and all 9 CSS strings
from `git show HEAD:.../scene.rs` and from the new files and found every one
**byte-identical**, with the item set unchanged.

`#[path]` on each submodule declaration is load-bearing, not decoration:
`scene.rs` is itself pulled in by `#[path = "uikit/scene.rs"] mod uikit;` from
three example binaries, so rustc would otherwise look for the children in
`examples/uikit/scene/`. A comment above the declarations says so.

Nineteen private gallery builders and `gallery` itself were undocumented in the
original; each got a one-line doc in its new home, which is the whole of
`component_gallery.rs`'s growth over the block it came from. Eight cross-file
intra-doc links would have silently rotted on the move and were retargeted with
identical rendered text. `needless_raw_string_hashes` fired on 8 of the 9 CSS
constants once they moved — it had been suppressed under one big file — and was
fixed in their new homes with the string bytes re-verified unchanged.

## The CPU renderer, along the "what one paint command actually does" seam

`crates/adapters/obscur-render-cpu/src/renderer/cpu_renderer.rs`,
**1,672 → 669** non-test lines.

The file exported one entity plus twenty-four free functions and six constants
forming six independent units, and the seam was in the call graph rather than the
line count: the `PaintCommand` arms were thin and every arm delegated to a body
with its own vocabulary. Seven entities left, each beside its parent in the
folder §3.3 already gave it: `brush_fill.rs` (273, the premultiplied-stop
constraint css-color-4 §12.3 imposes on every gradient), `glyph_raster.rs` (228,
the outline/fallback decision and the §5.18 synthesis hints that must apply
identically down either branch), `image_tiling.rs` (194),
`shadow_approximation.rs` (165, the two CSS Backgrounds 3 §6.1.1 spread signs,
with the alpha fade as the only actual approximation), `skia_lowering.rs` (131,
now the **only** place a domain value crosses the adapter boundary),
`clip_mask.rs` (114, the single rule a `PushClip` and a layer's own clip must both
enter through, or the two backends disagree) and `group_compositing.rs` (54).

The move is byte-faithful: a line-by-line comparison of the removed production
text against the new files' shows every algorithm line reappearing unchanged, the
only textual differences being new module docs, new imports and two doc lines
rewrapped around a disambiguated intra-doc path. Nothing was re-exported because
everything moved was already private, and all of it is `pub(crate)`, so no
tiny-skia type escapes the adapter. One test that called a moved function
directly moved with its subject; 26 unit tests were added for the extracted
entities' narrow interfaces.

`cpu_renderer.rs` is now **669 non-test lines against 1,507 test lines** — 69%
test — and no extraction of this kind can move that half, because the remaining
tests drive `CpuRenderer::render`, its public API.

# Closing the books: the bar becomes a check

## What the workspace measures

Measured across all **1,813** `.rs` files in the workspace with the campaign's
own rule — every line above the trailing `#[cfg(test)] mod tests {`, nothing
below it — **exactly one file is over the 1,000-line bar**:

| non-test | total | tests | file |
| --- | --- | --- | --- |
| 5,549 | 15,514 | 9,965 | `crates/application/obscur-runtime/src/runtime/ui_runtime.rs` |

The runners-up are 986 (`examples/wireframe/scene.rs`), 984
(`runtime/resource_bridge.rs`), 938 (`examples/showcase_window.rs`), 895
(`tests/visual_css.rs`) and 891 (`examples/windowed_counter.rs`) — every one of
them under the bar. `gradient_lowering.rs` is 906 lines *total* and 647 of code:
it is the largest file the property-compiler split produced, and it is the reason
the rule counts the way it does, since a naive total-line cap would have flagged
a file that is a third unit tests.

## The scoreboard

Before-counts are each stage's own measurement, checked here against
`git show HEAD:` with the same counter; after-counts are measured on the tree as
it stands.

| target | seam | non-test before → after |
| --- | --- | --- |
| `runtime/ui_runtime.rs` | state, then behaviour | 9,761 → **5,549** (7,272 at the file-size campaign's start) |
| `compiler/property_compiler.rs` | helper fan-in | 4,467 → **514** |
| `orchestration/scene_coordinator.rs` | painters | 4,281 → **803** |
| `orchestration/layout_coordinator.rs` | pipeline pass | 2,779 → **518** |
| `examples/uikit/scene.rs` | one stylesheet, one design system | 2,725 → **95** |
| `algorithm/grid_layout.rs` | W3C spec section | 1,891 → **636** |
| `algorithm/flex_layout.rs` | W3C spec section | 1,615 → **695** |
| `renderer/cpu_renderer.rs` | what a paint command does | 1,672 → **669** |
| `layout/paragraph_layout.rs` | pipeline stage | 1,626 → **598** |
| `computed/style_resolver.rs` | one fact per property | 1,167 → **429** |
| `property/property_id.rs` | one fact per property | 1,084 → **725** |
| `runtime/resource_bridge.rs` | thread boundary | 1,077 → **984** |
| `renderer/vello_renderer.rs` | GPU submission boundary | 1,052 → **694** |
| `conversion/scene_converter.rs` | mapping vs derivation | 1,037 → **639** |
| `parser/stylesheet_parser.rs` | lexical boundary | 1,039 → **607** |

Thirteen of the fifteen came under the bar. `resource_bridge.rs` came under it
with 16 lines to spare and its refusal is recorded in *The eighth pass*.
`ui_runtime.rs` did not, and its shortfall is argued rather than conceded.

## The one reasoned shortfall, so nobody re-litigates it

`ui_runtime.rs` stops at 5,549 non-test lines, and the arithmetic is the
argument, not an excuse:

- **~200 documented public methods** of `UiRuntime`'s own API, ~22 lines each and
  mostly doc comment. The campaign's standing rule is that the public API does not
  change and its documentation stays where a caller reads it, so these are pinned.
- **`run_frame` at 578 lines and `handle_pointer` at 331** — the two places
  everything is integrated, each reading or writing essentially the whole 47-field
  struct. Neither has a sub-algorithm left that does not need the whole struct.
- **~600 lines** of struct declaration, imports and `Debug`.

That floors the file near **4,000–4,500** before any judgement is involved.
Reaching 1,000 means deciding that `UiRuntime`'s public surface should be several
types — an API design decision, taken deliberately and with its callers in view,
not a file split. The second pass named what it deliberately left for a future
reader with that decision in hand: the text-edit key resolution (`try_text_edit`
and its four helpers, ~300 lines), the pointer gesture arming (~290, already thin
over `PointerSessions`), and the editing orchestration.

## The check `xtask` now runs

Two rules changed so the bar cannot drift back.

**`tools/xtask/src/rule/file_size.rs`** capped **total** lines at 2,000 with a
seven-entry ledger. It now measures **non-test** lines and caps them at
**1,000**. Non-test because §3.1 does not merely tolerate an entity's unit tests
in its file, it *requires* them there: most reach private items and could not
move, and counting them would push a well-tested entity over the bar for being
well tested. The ledger holds one entry.

**`tools/xtask/src/rule/main_entity.rs`** is new, and closes the last unimplemented
bullet of §3.16 — every other bullet already had a rule. A **main entity** is a
`pub` / `pub(crate)` `struct`, `enum` or `trait` declared at file scope **that
carries its own behaviour**: 60 or more lines of inherent `impl`, or of trait
body. The behaviour qualifier is not a softening, it is what makes the check say
anything about this workspace, and its threshold was measured rather than chosen:

- **194 of 1,813 files** declare more than one `pub`/`pub(crate)` type at file
  scope;
- **169** of those are re-exported by name from their own `mod.rs`, so the bare
  reading of "экспортирует несколько сущностей" catches them too;
- **4** hold more than one type with real behaviour.

The 190 in between are §3.1-legal and always were, because an entity's own
*vocabulary* is not a second entity: `font_face.rs` holds `FontFace` and the
`FontMetrics` its methods return, `table.rs` holds `Table` with the `Column` and
`SortDirection` a table is described in, and every `port/*.rs` holds a trait with
the `Null…`/`Mock…` stub that exists only to implement it. A check reporting all
194 would report nothing — it would be switched off within a week and the four
real cases would be invisible inside the noise. Behaviour is the right
discriminator because it is what this campaign found every time it hit the
violation: `paragraph_layout.rs` was not at fault for `LineStrut` being `pub`, it
was at fault because `LineStrut` had its own CSS 2.1 §10.8.1 formula, its own
`impl`, and an interface far narrower than its internals.

§3.16 asks for the check "без явного exception marker", so a file claims the
exception with an ordinary comment — `// xtask:allow(multiple-entities) — …` —
**and** an entry in the rule's `KNOWN` ledger. Both are required and both are
checked for staleness, so an exception cannot quietly outlive its reason. The
four files that claim it:

| file | why |
| --- | --- |
| `okui-compiler/src/compile.rs` | the orchestrator and the `ContractRegistry` it resolves names through — the one input `Compiler` cannot take by argument |
| `okui-compiler/src/validation/contract_validator.rs` | §3.11 names the validator's file and gives `ComponentContract` none; a contract's meaning *is* what the validator does with it |
| `obscur-geometry/src/primitive/corner_radii.rs` | §3.3 names `corner_radii.rs` and no `corner_radius.rs`; the set of four is defined over the quarter-ellipse pair |
| `window-winit/src/backend/winit_event_loop.rs` | `UserEvent`, `BackendState` and `WinitAppHandler` share one `Mutex`; separating them puts the lock's two halves in two files |

## The proof that both rules fire

A rule that has never failed is a rule nobody has tested. Both were driven into
every failure direction and then restored:

- `file_size` with the cap temporarily at **900**: reports
  `resource_bridge.rs` (984), `wireframe/scene.rs` (986) and
  `showcase_window.rs` (938) as over-cap with no ledger entry.
- `file_size` with the cap temporarily at **6,000**: reports `ui_runtime.rs`
  (5,549) as *under* the cap and demands its ledger entry be removed — the
  direction that stops a shrunken file growing back through the hole its entry
  left.
- `main_entity` with the marker stripped from `corner_radii.rs`: reports the file
  as on the ledger with no marker.
- `main_entity` with the behaviour threshold temporarily at **50**: reports four
  further files (`css_motion_rules.rs`, `menu.rs`, `text_stack.rs`, `perf.rs`) as
  exporting two main entities apiece, naming both.
- `main_entity` with the threshold temporarily at **150**: reports all four ledger
  entries as stale claims to be withdrawn.
- Cap and threshold restored: `cargo run -p xtask -- check` passes.

The rules carry twelve unit tests of their own, including the two traps a
line-based checker falls into here: a `pub(crate) mod tests` (several crates
write one so a sibling can borrow a fixture builder) and the braces inside the
raw-string CSS constants in `examples/uikit/*_theme.rs`, which would otherwise
put the rest of a file at brace depth 40. `main_entity` also refuses to be
exempted by its own source — it looks for the marker as a `//` comment line, not
as a substring, because the rule spells its own marker out in a `const`.

## The defects the campaign found, in one place

Fifteen instances of one recurring class, plus what else fell out. The class:
**a doc comment attached to the wrong item because a blank line went missing**,
so one item is silently undocumented and its neighbour's documentation opens by
describing something else. Every one was restored verbatim to its own item.

1–10. Found by the first campaign and the early passes, including
`size_replaced_elements` and `decode_one_image`.
11. `shape_text_lines` had **no** doc; its six-line block had fused onto
`align_width_for`'s, so the alignment-width helper's documentation opened by
describing paragraph shaping.
12. The removed `presses` field's 12-line doc still glued above `pointers`'s —
created *by* an earlier extraction rather than merely found by one. Its "keyed
rather than single" rationale existed nowhere; restored to
`PointerSessions::presses`.
13. The removed `commands` field's 22-line doc glued above `routing`'s. The half
with no other home — that a `UiRuntime` *is* one window surface, so a handler on
it is a §2.6 link-4 handler by construction — moved to
`HandlerRegistry::handlers`.
14. `insets_of` had no doc; its line was glued above `margin_edges_of`'s, which
calls itself its twin — which is exactly what hid the omission.
15. `measure` had no doc; its ten lines, including the **only** written
explanation of why measure and arrange are separate functions, were glued above
`overflow_for_text`'s. That argument is now also the head of
`measure_pass.rs`'s module doc.
16. The worst variant: `render_to_frame`'s seventeen-line doc **and** its
`#[allow(cast_possible_truncation, cast_sign_loss)]` had migrated onto the
two-line `cached_offscreen`. The shared core of every Vello render path was
undocumented, a cache helper advertised errors it cannot return, and the blur
casts were unsuppressed — the whole of that stage's 372 → 366 clippy delta.

Other defects, each fixed or recorded where it was found:

- **Duplicated logic hidden by a shared file.** `handle_scroll` open-coded
  `scrollable_container_at` exactly — the same filter, the same empty check, the
  same fold-to-innermost, eight identical lines existing twice, once public and
  once private.
- **A stale ledger that would have mispredicted the work.** Three `file_size.rs`
  entries were wrong on the evidence: "the arms are the entity" (the dispatch is
  212 of 4,467 lines), "the mapping is the extractable half" (the mapping is 690
  of 2,835), and `paragraph_layout.rs`'s "line breaking, bidi reordering and
  alignment share one line-box state" (they share a vocabulary, not a state).
  All three were rewritten to say what actually left.
- **Two `pub` entities in one file, stated by the file's own `mod.rs`.**
  `pub use style_resolver::{ResolutionContext, StyleResolver}`,
  `pub use property_id::{PropertyId, WritingMode}`,
  `pub use paragraph_layout::{LineStrut, ParagraphLayout}`, plus `ScrollPaint`,
  `MeasureCache`, `ArrangeCache` and `SceneSegment` — §3.16 in its plainest form,
  and now the thing `main_entity.rs` exists to catch before a reviewer has to.
- **A file with two test modules is a file with two subjects.** `property_id.rs`
  had `paint_only_tests` beside `tests`; it had named its own passenger.
- **Documentation that had gone false.** "one of the seven `PropertyId`
  longhands" (118); "multi-component shorthands are beyond the domain's value
  model"; `scroll_container_of` pointing at a `super::` item that does not exist;
  `spacing.rs` naming a stage (`ParagraphLayout::wrap_core` → `hang_line_trailing`)
  that had moved to `PackedLine::hang_trailing`.
- **A value spelled twice, sixty lines apart.** `INITIAL_FONT_SIZE: f32 = 16.0`
  and the `PropertyId::FontSize` arm's literal `16.0`; the extraction put them in
  one file and the arm now reads `Length::px(INITIAL_FONT_SIZE)`.
- **Near-duplicate navigation that only adjacency was checking.** `TokenCursor`
  and the stylesheet parser's private `Builder` carry byte-identical `skip_ws`
  and near-identical span readers. Reported, not changed: collapsing them mutates
  `pos` from several recovery walks and is a real change, not a refactor.
- **Broken and misdirected intra-doc links**, in the dozens, every one repaired
  with rendered text unchanged: two of them — `CornerRadius::circular_approximation`
  and `Compose::DestOut` — documented the Vello shadow backend's *known* losses
  and linked to nothing.

## What the next reader inherits

The bar is machine-enforced in both directions and the §3.16 check exists, so the
next file to cross either line will be reported by `cargo run -p xtask -- check`
rather than noticed. One file is over the size bar with its shortfall argued
above; four claim the entity exception with their reasons in
`tools/xtask/src/rule/main_entity.rs`. Nothing else in 1,813 files is over
either. The next reduction of `ui_runtime.rs` is not a refactor and should not be
attempted as one.

---

# The editing pass: the text tier, along the "decide, then act" seam

**Status:** done. **Four entities extracted** from `runtime/ui_runtime.rs`, which
went from **5,158 → 4,704 lines of non-test code** (15,131 → 14,681 total),
measured on the tree at the start and the end of this stage with the campaign's
own counter. The file is still over the bar and the shortfall is still the argued
one — see *The one reasoned shortfall*, above, which this pass does not overturn.

Suite **6,194 → 6,211 passed / 0 failed** — the 17 are the four new files' own
unit tests, and no existing test was weakened, rewritten, deleted **or moved**;
`clippy` **366 → 366**, with no warning in any file this pass created; `fmt`
clean; `xtask check` and `xtask consumer` both pass.

## Why this is not the refusal being re-opened

The first campaign refused to extract "editing" and the refusal stands: it asked
whether the cluster's **state** was one entity's state, found exactly one field
(`editing`) with no second field any operation must touch it with, and said so.
Nothing here disturbs that. `UiRuntime` still owns the `Option<TextEditSession>`,
still owns the orchestration around it, and the field count is unchanged.

This pass asks §3.16's other question — *how many main entities does this file
hold?* — of the ~500 private lines that tier occupies, and the answer is four,
each of which is a **decision** rather than a piece of state:

> **A group is an entity when it can answer its question without owning
> anything, and the acting is somebody else's job.**

That is the second pass's rule 5 (*report, do not act*) used as a seam-finder
rather than as a constraint on one already-chosen extraction. Every one of these
four reads a laid-out frame and a session and returns *what should happen*; every
one of them is now drivable with no runtime at all; and none of them holds the
scheduler, the handler registry, the scroll map or the session, which is why rule
2 does not veto any of them the way it vetoed `TextEditor`.

## Done

| Entity | Non-test lines | What it owns |
|---|---|---|
| `runtime/key_edit_resolution.rs` — `KeyEditResolver`, `KeyEdit` | 383 | The editable's keymap: a `KeyEvent` plus the held modifiers, against a laid-out field, resolved to one `KeyEdit`. The three motions that need the wrapped layout (vertical arrows, `Home`, `End`) are in here because each one's answer is a *fallback ladder* that only means anything as part of the keymap, and the multi-scalar grapheme cluster a `Backspace` must delete as one edit |
| `runtime/text_pointer_selection.rs` — `TextPointerSelection`, `TextPress` | 246 | Where a pointer lands in a field's text, and what a press or a drag there selects: the byte offset (the inverse of the caret-render advance walk), the word bracket, the line bracket, the drag range |
| `runtime/controlled_reseed.rs` — `ControlledReseed`, `ReseedDecision` | 135 | The controlled-input reseed gate — the one place the toolkit decides **who owns the text**, as three strings compared in one function |
| `runtime/compose_only_edit.rs` — `ComposeOnlyEdit` | 126 | Whether a mutating edit can be flushed with a compose-only frame, i.e. whether the field is box-stable: multiline, wrapping, width-dependent alignment and indefinite width each rule it out |

Four files, **890 lines of non-test code**. What is left in `ui_runtime.rs` for
this tier is the acting half — `apply_edit`, `after_edit`, `commit_editable_change`,
`sync_editing_to_focus`, the two caret-scroll seats and the `handle_ime` lifecycle
— and every one of those is orchestration against the routing registry, the
scheduler, the scroll map and the session at once, which is the first campaign's
refusal restated with its scope now visible.

## Why the keymap is one entity and not five

`try_text_edit` was 153 lines calling four helpers, and the tempting split is by
key family — motion here, deletion there. It is wrong, and the reason is in the
map rather than in the code: the keystroke → edit map has to be **total and
exclusive**. Total, because every key the field does not claim must fall through
to the shortcut router unchanged; exclusive, because a key resolved as an edit is
not also a shortcut and not also an activation. Both properties are properties of
the *whole table*, so the table is the entity, and the four helpers are §3.1's
"private local functions" of it — none is reachable from anywhere else, and each
one's interface to the keymap is a single number or a single command.

The one thing that did have to become explicit is the readonly gate. It sat
inside the `Backspace` arm as `!self.is_within_readonly(node) && …`, and a
resolver cannot ask it — the ancestor walk is the runtime's. So `KeyEdit::
DeleteGrapheme` carries the resolved cluster and the runtime applies the gate at
the same point in the same order it always did. The one difference is stated
rather than hidden: on a **readonly** field's `Backspace` the cluster is now
resolved and then discarded, where the old short-circuit never resolved it. It is
a pure read of the buffer with no observable effect, and it is the price of the
gate living where the tree walk does.

## What this pass found

**No new instance of the campaign's recurring doc defect. The counter stays at
sixteen.** As in the eighth and tenth passes this is a measurement rather than an
impression: a mechanical pass over every item in the four new files and in the
whole production half of `ui_runtime.rs` — every `fn`, `struct`, `enum`, `type`
and `const` — finds exactly one item with no doc comment directly above it, and
it is `handle_ime`'s function-local `enum ImeFollowUp`, which carries an ordinary
`//` comment and always did.

Three smaller things:

- **A doc link whose referent moved out from under it.** `caret_scroll.rs`
  describes `has_definite_px_height` as "the vertical analogue of
  [`has_definite_px_width`]" — a link that never resolved (the target was a
  private free function in another module) and that now names a function in a
  third file. It reads as plain text naming `compose_only_edit.rs`, so the
  sentence still says which twin it is the twin of.
- **Ten imports that only the moved code needed.** `TextAlignment`,
  `TextDirection`, `Length`, `ComputedValue`, `NavigationUnit`, `EditDirection`,
  `Modifier` and the `alignment_for`/`direction_for`/`white_space_for` line left
  `ui_runtime.rs` with their readers. `ComputedValue` came back **into the test
  module**, in the file's established style of saying which extraction took the
  production reader away.
- **Two `apply_edit` calls that were one.** The `Backspace` arm ended its
  cluster branch with `apply_edit(Delete)` and `return true`, and then fell
  through to an identical `apply_edit(Delete)` for every other case. Merged in
  the new shape, byte-for-byte the same sequence of calls in both branches.

## Where the open defect now lives

The first campaign reported, and did not change, that
`reseed_editing_from_retained` re-derives only the **horizontal** caret-follow
scroll where `after_edit` re-derives both axes — so an externally shortened
controlled `<textarea>` keeps a `scroll_top` computed for the old content. It is
still not changed, and it is now **more** visible rather than less: the gate's
decision moved to `ControlledReseed`, so the runtime's method is nine lines of
`match`, and the single surviving `self.update_caret_scroll()` sits under a
`NOTE` naming the asymmetry, the ledger row it belongs to, and the fact that
fixing it is its own change with its own test. Before, the same call appeared
twice inside a 60-line method and neither site said anything.

## What the next reader inherits

The second pass named three things it left behind. Two of them are now done —
the text-edit key resolution and the editing orchestration's decidable half — and
the third is unchanged: **the pointer gesture arming** (`arm_pan`, `update_pan`,
`arm_long_press`, `advance_holds`, `fire_long_press`, `launch_fling`,
`advance_flings`, ~290 lines), which is already thin over `PointerSessions` and
whose recognizers the first pass made pure, so the win there is smaller than the
line count suggests.

Past that, the shortfall argument above is still the whole of it: what remains is
`UiRuntime`'s documented public surface, `run_frame`, `handle_pointer`, and the
struct itself.

---

# The pointer pass: `handle_pointer`, along the "one step, one question" seam

**Status:** done. **Six entities extracted** from `runtime/ui_runtime.rs`, which
went from **4,704 → 4,537 lines of non-test code** (14,681 → 14,516 total),
measured on the tree at the start and the end of this stage with the campaign's
own counter. The file is still over the bar and the shortfall is still the argued
one — see *The one reasoned shortfall*, above, which this pass does not overturn
and this chapter's last section restates in the pointer path's own terms.

Suite **6,211 → 6,229 passed / 0 failed** — the 18 are the six new files' own
unit tests, and no existing test was weakened, rewritten, deleted **or moved**;
`clippy` **366 → 366**, with no warning in any file this pass created; `cargo doc`
reports no warning against any file this pass created or changed that it did not
report before; `fmt` clean; `xtask check` and `xtask consumer` both pass.

## Why the seam is different again

Every earlier pass asked its question of a *method list*: which fields share an
invariant (first), which private method is a self-contained sub-algorithm
(second), which decision can be made without owning anything (the editing pass).
Applied to what was left of `ui_runtime.rs` those questions are answered: 131 of
the 172 remaining methods are `pub`, and rule 3 pins them.

So this pass asks it of **one method's body**. `handle_pointer` is public and was
326 lines, and rule 3 pins its signature and its caller-facing doc — but not its
body, which is a *numbered sequence of steps* that the code itself had already
labelled `0`, `0b`, `1`, `1b`, `2`, `3`, `3a`, `3a-dnd`, `3b`, `3c`, `4`. A step
that needs a paragraph of comment to say which coordinate space it is in, or which
of three conditions makes a release a click, is not §3.1's "private local function
with no independent semantics". It is an entity whose file happens not to exist
yet.

The test this pass applies, which is the second pass's rule 1 pointed at a body
rather than at a method list:

> **A step is an entity when the comment that explains it is longer than the code
> that performs it, and the explanation is about the step rather than about the
> sequence.**

Both halves matter. The first half finds the candidates — a step whose *why*
outweighs its *what* is carrying knowledge nothing else in the file needs. The
second half rejects most of them: a comment explaining why step 3a comes before
step 3b is about the sequence, and the sequence is `handle_pointer`'s own subject.
That is why the press step and the release's acting half stay, and it is stated
below rather than left implicit.

## Done

| Entity | Non-test lines | What it decides |
|---|---|---|
| `runtime/pointer_landing.rs` — `PointerLanding`, `PointerLandingResolver` | 174 | Steps 0, 0b and 1: the host's physical position mapped to content space, folded through the scroll containers it lies within, and resolved to a target by capture-else-hit-test — carrying **both** points, because a clip is tested on screen while a box is tested in unscrolled layout space |
| `runtime/gesture_arming.rs` — `GestureArming` | 143 | Step 3a: which scroll container a direct contact can pan and which long press it can arm, resolved against the frame — the tree-side half of §6.16, as against the recognizers `PointerSessions` already owns |
| `runtime/structural_states.rs` — `StructuralStates` | 99 | The two reserved states that make a control inert: `disabled` (blocks the interaction) and `readonly` (blocks only the write), each answered for a node **and its element ancestors** |
| `runtime/pseudo_state_projection.rs` — `PseudoStateProjection` | 94 | Step 4: the hover chain, the focused node and every outstanding press projected into the per-node `PseudoStates` map the cascade matches `:hover` / `:focus` / `:active` against |
| `runtime/pointer_release.rs` — `PointerRelease` | 91 | Step 3b's decision: whether a release is a **click**, on what, and with what run length — its own press, the same node, and trusted |
| `runtime/hover_update.rs` — `HoverUpdate` | 80 | Step 2: the chain an event leaves hovered, and whether `:hover` now matches differently — including the rule that a lifted **finger** leaves nothing hovered while a lifted pen leaves the chain where the hit test put it |

Six files, **681 lines of non-test code** and **18 new unit tests**. As in the
second pass the arithmetic does not close: 681 lines moved out reduce the file by
167, because a step that leaves takes its explanation with it and leaves a
sentence naming where it went. `handle_pointer` itself went **326 → 267**.

### What the prize looks like this time

Every one of the eighteen tests constructs its subject from nothing: an empty
frame lands a point unchanged on nothing and a capture targets its captor
wherever the pointer is, a capture on a node that left the tree releases and
falls back, a mouse arms neither gesture, a disabled control disables everything
inside it while a readonly one gates only the write, focus lands on one node
where hover lands on the whole chain, two outstanding presses both light
`:active`, a lifted finger leaves nothing hovered where a lifted pen keeps
hovering, a second finger does not steal the first finger's release, and an
untrusted release clicks nothing but still ends the press.

That last group is the one worth naming. The three conditions that make a release
a click — *its own press*, *the same node*, *trusted* — were one conjunction in
the middle of a 326-line method, and **each of the three was, at some point, the
bug**: the single press anchor a second finger overwrote, the press that slid off
its control, and §2.3 step 4's trust gate that §9 forbids weakening. Two of them
fail silently (the control simply does not respond) and the third fails by
responding to input the platform never vouched for. All three are now four unit
tests that need no runtime, no view and no frame.

## What did not move, and why

The pass examined all eleven of `handle_pointer`'s labelled steps. Five moved.
The refusals are here so the next reader does not re-derive them:

- **Step 3, the press** (focus moves, the editing session re-seats, a newly
  focused field scrolls into view, the press anchor records, the caret seats).
  Rule 2 vetoes it: a bundle for this step names `focused`, `editing`, `scroll`,
  `pointers`, `tree`, `boxes`, `computed` and the scheduler — the struct with an
  extra indirection, which the first pass's ledger already refused under that
  name. What *could* be lifted out of it was the disabled gate, and it was
  (`StructuralStates`).
- **Step 3b's acting half** — resolve the activation command, run it with the
  device envelope, fire the virtual row's click handler. Same veto, and the
  decision half came out on its own (`PointerRelease`), which is the shape the
  editing pass established: decide, then act.
- **Step 3a-dnd, the in-flight data drag**, and `finish_drag`. These consume the
  pointer, commit a transaction through the enactor, and request frames; the
  "report" version returns a three-way verdict the runtime immediately switches
  on, which is an indirection rather than an entity.
- **The gesture *advance* half** — `update_pan`, `advance_holds`,
  `fire_long_press`, `launch_fling`, `advance_flings`. The second pass and the
  editing pass both said the win here is smaller than the line count suggests,
  and measuring it confirms them: the five methods are **141 lines, of which 54
  are doc comment and 75 are code** — five signatures, five closing braces, and
  bodies of three to twelve lines each that call `PointerSessions` and then apply
  what it returned with `scroll_by` or a command dispatch. The recognizers are
  already pure and already extracted; what is left is the application, and
  application is the runtime's.
  This pass therefore took the *arming* half, which is the part that consults the
  frame — a scrollable container under the finger, a holdable row, an
  `EVENT_LONG_PRESS` binding, a disabled ancestor — and left the advance half
  where it is. That is the boundary the third bullet of *What the next reader
  inherits* was pointing at, now drawn.

Two smaller ones, for completeness: `activate`, `dispatch_command_from` and
`dispatch_command_event` (143 lines, ~120 of them doc) are already thin over
`CommandDispatch`, and their docs describe the runtime's own contract — where
§2.3's four steps meet §2.6's chain — rather than an algorithm that could move.
And the resource fan-out (`resolve_element_images`, `resolve_background_images`,
`resolve_element_vectors`, `mark_image_visibility`, `commit_resource_results`) is
five delegating calls into `ResourceBridge`; the only real duplication in it, the
`ResourceReporting` literal built three times, cannot become a `&self` helper
because the three call sites need `&mut self.resources` at the same time.

## What this pass found

**No new instance of the campaign's recurring doc defect. The counter stays at
sixteen.** As in the eighth, tenth and editing passes this is a measurement: a
mechanical pass over every item in the six new files and in the whole production
half of `ui_runtime.rs` — every `fn`, `struct`, `enum`, `type` and `const` —
finds exactly one item with no doc comment directly above it, and it is still
`handle_ime`'s function-local `enum ImeFollowUp`, which carries an ordinary `//`
comment and always did.

Three smaller things:

- **A private helper whose last two readers left together.** `hover_chain_for`
  was a one-line wrapper over `AncestorChain::of(...).into_vec()` with three
  callers. Two of them — the disabled/readonly gates and the `:active` walk —
  left with `StructuralStates` and `PseudoStateProjection`, both of which call
  `AncestorChain` directly, so the wrapper is gone and its one remaining caller
  spells the walk out. A wrapper with one caller is the caller's line.
- **`ActiveHold` was not re-exported.** `PointerSessions` has exported
  `ActivePan`, `PointerCompletion` and itself since the first pass, but the hold
  it builds in `new_hold` was reachable only because the one caller lived in the
  same file that constructed it. Naming it in `mod.rs` was a prerequisite for
  `GestureArming` returning one, and it is the kind of gap that only shows up
  when something outside tries to hold the value.
- **Three imports that only the moved code needed.** `HoverTransition`,
  `PointerDevice` and `AncestorChain` left `ui_runtime.rs` with their readers.
  `PointerDevice` came back **into the test module**, in the file's established
  style of saying which extraction took the production reader away.

## What is left, and what it is not

`ui_runtime.rs` is **4,537 lines of non-test code** (14,516 total; 9,979 of them —
69% — are its `#[cfg(test)] mod tests`). Measured, that production half is:

- **2,513 lines across 131 `pub` methods** — 55% of the file, pinned by rule 3.
  A caller reads `run_due_frame`'s contract on `run_due_frame`, and the average
  public method here is 19 lines of which the body is a handful.
- **1,392 lines across 41 private methods**, and `run_frame` alone is 330 of
  them. Excluding it, the median private method is 24 lines; the largest are the
  editing tier's acting half (`try_text_edit`, `after_edit`, `apply_edit`,
  `commit_editable_change`), the command wrappers (`activate`,
  `dispatch_command_from`), this pass's own `arm_gestures`, and the input-queue
  drain — all refused above or by the editing pass, each for a reason written
  down.
- **632 lines of struct, field docs, imports, `is_activation_key` and the
  `Debug` impl.**

So the private half this stage inherited — 1,491 lines when it started — is now
1,392, and 330 of that is the one method every pass has refused. The remaining
~1,060 lines of private behaviour are not one undiscovered entity; they are
forty methods averaging 26 lines apiece, each already thin over an entity that
was extracted in an earlier pass.

The shortfall argument is therefore unchanged and is now also true of the pointer
path specifically: reaching 1,000 lines requires deciding that `UiRuntime`'s
public surface should be several types rather than one. That is an API change and
a design question, and answering it inside a refactoring pass is exactly what this
campaign's method exists to prevent.

# The audit pass: the ledger checked against the tree, and a proposal instead of a shrug

This pass extracted nothing. Three stages had just run against `ui_runtime.rs` in
sequence — the frame pass, the editing pass, the pointer pass — each reporting its
own numbers, and the debt register in `tools/xtask/src/rule/file_size.rs` still
quoted the figure the stage *before* them had left. A ledger that is out of date
is a claim about the workspace that is no longer true, so this pass re-measured
everything from the tree itself, corrected the register, and wrote down the one
decision the campaign cannot take on the owner's behalf — with its arithmetic and
its price, rather than with the shrug the last three chapters ended on.

## What the workspace measures now

Measured across all **1,829** `.rs` files with the campaign's own rule — every
line above the trailing `#[cfg(test)] mod tests {`, nothing below it — **exactly
one file is over the 1,000-line bar**, and it is the same one:

| non-test | total | tests | file |
| --- | --- | --- | --- |
| 4,537 | 14,515 | 9,978 | `crates/application/obscur-runtime/src/runtime/ui_runtime.rs` |

Sixty-nine per cent of that file is its `#[cfg(test)] mod tests`, which the rule
does not count and §3.1 requires to be there. The runners-up are 986
(`examples/wireframe/scene.rs`), 984 (`runtime/resource_bridge.rs`), 938
(`examples/showcase_window.rs`), 906 (`compiler/gradient_lowering.rs`) and 895
(`tests/visual_css.rs`). Five files sit within 120 lines of the bar and none is
over it; the field behind the exception is tight, which is the point of keeping
the register honest.

The 4,537 is not this pass's arithmetic: `xtask check`, with the ledger entry
temporarily removed, reports the same number back.

## The chapter the frame pass did not write

Between *Closing the books* and *The editing pass*, a stage ran that took
`run_frame` from 568 lines to 301 and `ui_runtime.rs` from 5,549 to 5,158, and it
never got a chapter. It belongs in the ledger, so here it is, verified against the
files it created:

- **`reconcile_phase.rs`** — `ReconcilePhase` (268 lines). Step 1 of §3.4:
  materialize the desired view *after* dispatch, realize the §4.9 virtual
  collections, commit the root reconcile through the transaction boundary, mount
  and render the components, report the restyle roots.
- **`style_phase.rs`** — `StylePhase` (258). Steps 2–3b: the cascade list, the
  match and resolve through the already-extracted `RestylePhase`, §4.16's
  generated content, the `StyleKey` warrant, and §4.17's motion overlay.
- **`arrange_phase.rs`** — `ArrangePhase` (223). Step 4: layout styles plus their
  three patches, the grid templates, the incremental arrange with its two caches
  and its invalidation set, the hit index.
- **`compose_phase.rs`** — `ComposePhase` (203). Step 5: compose, the presentation
  overlay, the inspect highlight, and `scaled_root`.
- **`desired_view.rs`** — `DesiredView` (18). The tree a frame reconciles,
  `Ready` or `Build`, resolved lazily so an input handler that commits state runs
  before the view is rebuilt from it.

Each is a **borrow bundle**: every field borrows state the runtime owns across
frames, taken for one phase and released with it. That is the same shape the state
pass established and the reason these five sit in `runtime/` beside the frame's
already-extracted per-phase machinery (`restyle_phase.rs`,
`component_render_pass.rs`, `layout_classifier.rs`, `frame_plan.rs`,
`frame_phase_clock.rs`) rather than in `frame/`, which holds the frame's *public*
vocabulary.

That stage also found **defect #11 of the campaign's recurring class**:
`UiRuntime`'s entire 41-line doc block — spec §1.4 and §3.4–§3.7 plus the whole
*Relationship to `HeadlessHost`* section — was attached to the private
`enum DesiredView` below it, because the blank line between the two items was
missing. Rustdoc had been rendering the runtime's architecture documentation on a
two-variant private enum. Moving `DesiredView` to its own file forced the fix.

With this chapter written, the campaign's tally against `ui_runtime.rs` is **41
entities across five passes** — twelve along the state seam, fourteen along the
behaviour seam, five frame phases, four in the text tier, six pointer steps — and
all 41 live beside it in `runtime/`, which now holds 62 files.

## What the 4,537 lines actually are

Every number below is measured on the tree as it stands, not inherited:

| region | lines |
| --- | --- |
| module doc + 65 `use` lines | 72 |
| `UiRuntime`'s own doc block | 76 |
| `struct UiRuntime` — 47 fields with 255 lines of field docs | 305 |
| `impl UiRuntime` | 4,055 |
| `is_activation_key` + `impl Debug` | 29 |
| **total** | **4,537** |

Inside the impl:

| | count | lines | of which doc/attr |
| --- | --- | --- | --- |
| `pub` methods | 131 | 2,543 | 1,325 |
| private methods | 40 | 1,279 | 385 |
| impl scaffolding (header, brace, section comments, blanks) | — | 233 | — |

And by line *kind*, across the whole non-test half: **2,015 doc-comment lines,
481 inline-comment lines, 227 blank, and 1,814 lines of code.** Forty per cent of
this file is code. Fifty-five per cent is prose a caller reads, and rule 3 of this
campaign — the public API and its documentation do not move — pins the larger part
of it exactly where it is.

The single arithmetic fact that settles the file-split question: **delete every
private method, all forty, including `run_frame`, and 3,258 lines remain.** No
extraction of private behaviour can reach 1,000, because private behaviour is not
what the file is made of.

## The gap to 1,000, in measured lines

3,537 lines have to go. They can only come from:

- **1,279** — all 40 private methods. `run_frame` is 319 of them; the next
  largest are `try_text_edit` (72), `dispatch_command_event` (55, of which 42 are
  doc), `after_edit` (53), `update_pan` (50), `commit_editable_change` (46) and
  `finish_drag` (44). Every one of these has been examined by an earlier pass and
  refused with a reason: each is already thin over an entity that was extracted,
  and a bundle for it would name most of the 47-field struct.
- **2,543** — the 131 public methods. Rule 3 forbids moving them *as a
  refactoring*. Moving them is possible only as an API change.
- **715** — the struct, its field docs, the type doc, the imports, `Debug`. This
  shrinks only if the struct's fields are grouped into sub-structs.

So: 1,279 + 2,258 of the public 2,543 = the 3,537 needed. **Eighty-nine per cent
of the public surface would have to leave the file.** That is not a file split by
any definition this campaign has used; it is a redesign of what `UiRuntime` *is*.

## The proposal: thirteen facets, and what they cost

Since the owner has to make this decision, here it is as a concrete design with
numbers rather than as an observation that a decision exists.

**The shape.** `UiRuntime` keeps its state and its integration. Each cohesive
group of its public API becomes a **facet** — a `pub struct XxxFacet<'a>(&'a mut
UiRuntime)` in its own file in `runtime/`, reached by one accessor
(`rt.motion()`, `rt.scrolling()`, `rt.commands()`). The facet holds the methods
*and their doc comments*, which is the only way the doc lines actually leave the
file. This is the same borrow-bundle shape as the five frame phases — a borrow of
runtime state taken for the duration of a call and released with it — applied to
the public surface instead of to a frame step.

**The partition.** All 171 methods, 3,822 lines, assigned exhaustively (no method
appears twice, none is left over):

| facet | lines | pub methods | private methods |
| --- | --- | --- | --- |
| `PointerFacet` | 638 | 13 (397) | 9 (241) |
| `EditingFacet` | 557 | 6 (152) | 12 (405) |
| `CommandFacet` | 477 | 20 (338) | 5 (139) |
| `FrameFacet` | 337 | 13 (262) | 4 (75) |
| `ScrollFacet` | 266 | 13 (248) | 1 (18) |
| `ResourceFacet` | 202 | 10 (149) | 6 (53) |
| `MotionFacet` | 153 | 6 (153) | — |
| `SurfaceFacet` | 150 | 12 (150) | — |
| `TypographyFacet` | 132 | 6 (132) | — |
| `AccessibilityFacet` | 117 | 5 (88) | 2 (29) |
| `KeyboardFacet` | 108 | 1 (108) | — |
| `DiagnosticsFacet` | 86 | 6 (86) | — |
| `FocusFacet` | 60 | 5 (60) | — |
| **facets, total** | **3,283** | **116** | **39** |
| core: `new`, the view/stylesheet/component sources, the output readers, `run_frame` | 539 | 15 (220) | 1 (319) |

Every facet is under the bar with room to spare; the largest, `PointerFacet`, is
638 lines and is `handle_pointer` (285) plus capture, drag and the gesture-advance
half that the pointer pass measured and left.

**What `ui_runtime.rs` would then be.** 539 method lines + 715 structural lines =
**1,254** — still 254 over. The last 254 come from one further move: `run_frame`
(319) becomes a `FrameRun` bundle over the five phases that already exist, leaving
the file at **~935 lines**: the struct, its field docs, the type's doc block, the
imports, `new`, the view and stylesheet sources, the output readers, `Debug`. That
is a file that finally holds one entity — the runtime's *state* — and it meets the
bar.

**The price, measured.** The method names in this partition appear as `.name(`
call sites **2,842 times across 253 files** in this workspace (2,512 if the names
that collide with other types' methods — `id`, `viewport`, `locale`, `hovered`,
`focused`, `tree`, `scene`, `animate`, `snapshot` — are excluded); 1,278 of them
are inside `ui_runtime.rs`'s own test module. Every one becomes
`rt.facet().name(...)`. Keeping the old spelling with forwarding methods is the
one option that must be rejected outright: a forwarder plus its doc comment is
~19 lines, so forwarding all 116 public methods puts back 2,200 lines and the file
does not move at all. **The facet split only works if the call sites change**, and
that is precisely the part that is the owner's to decide, not a refactoring pass's.

**The smaller, cheaper variant.** If changing 2,800 call sites is not acceptable,
grouping only the *state* — the 47 fields into ~8 sub-structs — takes about 200
lines off the struct and its docs and changes no caller. It does not reach 1,000
and it never will; it is worth naming only so the choice is not presented as
all-or-nothing.

## The register

`tools/xtask/src/rule/file_size.rs` now says 4,537 rather than 4,704, names all
five passes instead of three, and carries the composition above so a reviewer can
check the claim without opening the file. The rule itself is unchanged: one entry,
still deserved, and the rule would flag the entry as stale the moment the file
came under the cap. It does not fire today, and this pass verified that it fires
in both directions.

Verification for this pass: `cargo fmt --all --check` clean, `xtask check` and
`xtask consumer` pass, the suite is 6,229 passed / 0 failed, clippy 366 — every
number unchanged from the pointer pass, because nothing but a doc comment, a
ledger string and this chapter changed.

---

# The harness pass: the test half, along the "arrange, or assert?" seam

**Status:** the harness exists and one theme is migrated onto it. `ui_runtime.rs`
went from **12,734 lines to 11,641**, its `#[cfg(test)] mod tests` from **9,979 to
8,886**, and its `#[test]` count from **265 to 231**. The 34 that left are in
`crates/application/obscur-runtime/tests/touch_gestures.rs`, unchanged. The
non-test half is **2,755 lines, exactly as before** — this pass deliberately does
not touch it, and the `file_size` register says so.

Suite **6,229 passed / 0 failed** (unchanged: 34 tests changed address, not
existence), `clippy` **366** (unchanged), `fmt` clean, `xtask check` and
`xtask consumer` both pass.

## Why the seam is different again

Every previous pass asked a question about *production* code — which fields share
an invariant, which behaviour has its own vocabulary — and the audit pass closed
by measuring that **69% of this file is its test module**, which §3.1 requires to
live beside its entity and which no extraction of production code can move.

But "beside its entity" is a rule about *tests*, and this file's test module is
not only tests. Of its 9,979 lines, **1,128 were helpers** — 103 functions, 11
types, 13 constants — and a helper is not a test. It is either an **arrangement**
(build a view, a sheet, a pointer event, a runtime already in some state) or a
**reading** (which node is this, how many images did that paint). Neither
assertion nor subject; §3.1 does not put either of them in this file.

crate-arch §2.4 already names where they go, and had done since before the
campaign started: `obscur-testkit` — *semantic selectors; fixtures; mock
providers; snapshot normalization; high-level assertions; plugin test host
facade.* **Производственная логика здесь не живёт.** The crate existed as a
**49-line skeleton**: eleven files holding five unit structs with a doc comment
each, no dependencies, and `docs/api/12-reality.md` recording it as "unreachable
by construction".

## What the crate now holds

Eleven entities, in the folders §2.4's charter already named:

| file | entity | what it arranges or reads |
|---|---|---|
| `fixture/test_fixture.rs` | `TestFixture` | the one `RuntimeId` and the one 300×300 viewport every fixture shares, and a bare runtime at both |
| `fixture/class_rule.rs` | `ClassRule` | a `StyleRule` selecting one class, optionally `:hover`/`:focus`, with the declaration vocabulary the fixtures need (`sized`, `background`, `scrollable`) |
| `fixture/interaction_panel.rs` | `InteractionPanel` | the 100×100 panel over two stacked 100×50 children, each bound to its own click command — the fixture two pointers can work at once |
| `fixture/long_press_panel.rs` | `LongPressPanel` | the same panel with `a` click-bound and `b` click- *and* hold-bound, so a gesture is told apart by which handler ran |
| `fixture/scrolling_list.rs` | `ScrollingList` | the 200×120 `overflow: auto` scroller over six 40px rows — 120px of scroll, which every scroll assertion in the suite is written against |
| `fixture/mouse_event.rs` | `mouse`, `mouse_at` | the trusted mouse contact, with and without an explicit clock |
| `fixture/touch_contact.rs` | `finger`, `finger_at`, `untrusted_finger` | the touchscreen contact, likewise, plus the untrusted form §2.3 step 4 gates on |
| `selector/node_path.rs` | `NodePath` | the node at a path of child indices below the root, which is what "root → panel → field" was being spelled as, three lines at a time |
| `assertion/painted_images.rs` | `PaintedImages` | how many images a display list paints, and the tile the single one occupies |
| `mock/canned_image_loader.rs` + `mock/canned_image_decoder.rs` | `CannedImageLoader`, `CannedImageDecoder` | a provider pair that answers the same way every time, or fails every time |
| `mock/variant_image_loader.rs` + `mock/variant_image_decoder.rs` | `VariantImageLoader`, `VariantImageDecoder` | a fake filesystem of DPI variants that records **which paths were asked for** — usually the sharper half of the answer |

Three placeholder stubs were withdrawn rather than left beside the real entities
they were standing in for: `TreeAssertion`, `MockProvider`, and (see below)
`SemanticSelector`. `PluginTestHost` stays, because nothing drives plugin
activation through a harness yet and the stub is an honest statement of that.

## The constraint that shaped everything: `UiRuntime` is built twice

The plan for this pass assumed the harness could serve **both** the unit tests in
`src/` and the integration tests in `tests/`. Half of that is false, and the
compiler says so in one line:

> note: there are multiple different versions of crate `obscur_runtime` in the
> dependency graph

It is not a dependency cycle — cargo is perfectly happy, and `xtask`'s graph rule
skips dev-dependencies by construction. It is that a crate's **lib** target and
its **lib-under-`--test`** target are two distinct crate instances. `obscur-testkit`
links the first; `ui_runtime.rs`'s `#[cfg(test)] mod tests` *is* the second. A
`&UiRuntime` cannot cross between them.

So the harness splits in two by what its signatures name:

- **Crosses the boundary** — anything whose types come from a crate built once:
  `ClassRule` (a `StyleRule`), `mouse`/`finger` (a `PointerEvent`),
  `PaintedImages` (a `DisplayList`), the four mock providers (traits from
  `obscur-resources`), `InteractionPanel::view()`/`sheet()` and
  `ScrollingList::view()`/`sheet()`/`with_rows()` (a `View`, a `Stylesheet`), and
  `TestFixture`'s two constants. The unit tests that stayed use all of these.
- **Does not** — anything naming `UiRuntime`: `TestFixture::runtime()`,
  `NodePath`, and the fixture *structs* themselves. Those are reachable from
  `tests/` only.

Which means the tree-walking selectors (`counter_child`, `panel_nodes`,
`scroller_nodes`, `panel_single_child`) and `render_resolved` are still written
out in `ui_runtime.rs`, and each now says why in a comment. **This is not a
workaround to be found later — it is a standing argument for the migration**: a
test that moves to `tests/` gains the whole harness, and a test that stays gets
only the value builders.

Two entities were built, found to have no caller that could reach them, and
**withdrawn rather than left as dead public API** — the campaign's rule 4 applied
to the harness:

- `render_resolved` (the "render → drain → render" normalization §2.4 calls
  snapshot normalization). Its callers are the nine image tests, every one of
  which stays in `src/` this pass, and its signature names `UiRuntime`. It
  belongs to whichever stage migrates the image theme.
- `SemanticSelector` (nodes by class). Its one caller is `count_class`, whose
  three virtualization tests likewise stay. Same fate, same reason.

## Why the theme is touch gestures

Four themes were measured as candidates. The image/resource theme is the one that
would have exercised all five of §2.4's groups at once — and it is the one that
**cannot** move: seven of its twelve tests read `runtime.resources`, a private
field, through a `resident_image` helper, and one of those reads
`ResourceBridge::state_of`, which is `#[cfg(test)]` by an earlier pass's own rule
4. Widening the public API to move a test is exactly what this campaign forbids,
so five movable tests out of twelve is not a theme; it is a fragment.

Touch gestures is 34 tests that drive the runtime entirely through its public API
— press, move, lift, tick, read the scroll offset — and what they have in common
is a **contact**, not a subsystem: a finger that presses and waits is a long
press, one that travels is a pan, one that travels fast and lifts is a fling, one
taken away mid-gesture is a cancel. Nearly every assertion is about what a gesture
must **not** also do, which is the same failure four times over and belongs in one
file.

## What moved, and what changed in the moving

Every one of the 34 keeps its name, its assertions and its intent. What changed is
the fixture line at the top:

```rust
let (mut runtime, _view, _sheet, _a, _b, clicked, held) = long_press_fixture();
// becomes
let LongPressPanel { mut runtime, clicked, held, .. } = LongPressPanel::new();
```

The tuples were the reason three helpers carried `#[allow(clippy::type_complexity)]`.
Destructuring a named struct keeps every following line byte-identical — which is
the point: the diff should be readable as "the fixture changed shape", never as
"an assertion changed".

Two smaller rewrites, both in the same spirit: the one test that authors its own
sheet now says `ClassRule::new("scroller").sized(200.0, 120.0).scrollable()`
instead of pushing a keyword declaration onto a `Vec`, and the untrusted-contact
test says `untrusted_finger(...)` instead of spelling a `PointerEvent` out to
avoid the trust bit.

## What this pass found

- **A duplicated helper.** `finger_t` and `finger_at` were the same function under
  two names — `finger(...).with_timestamp(t)` — defined 800 lines apart, one for
  the multi-click tests and one for the fling tests. There is now one,
  `finger_at`, and the three `finger_t` call sites read it.
- **Two fixture fields no caller has ever read.** All ten call sites of
  `long_press_fixture` spelled its 4th and 5th elements `_a, _b`. They are kept on
  `LongPressPanel` as `tappable`/`holdable`, because `HOLD_TARGET_Y` is
  meaningless without the child it points at — but the tuple had been carrying
  them unread since it was written.
- **No misplaced doc comment.** The class of defect this campaign found eleven
  times does not appear in the test module: helpers there use ordinary `///`
  comments and are short enough that a missing blank line has nowhere to hide.

## What the next reader inherits

The pattern is proved and four themes are left. In rough order of how cleanly they
come out:

1. **Pointer/hover/focus dispatch** (~20 tests) — already on `InteractionPanel`'s
   view and sheet; needs `panel_nodes` to become `InteractionPanel::nodes` at the
   call site, which only works in `tests/`.
2. **Command routing and event listeners** (~30) — the five-link chain, the phase
   walk, `stopPropagation`. Public API throughout; needs a `two_row_view` fixture
   and the `PhaseLog`/`KeySeen` recorder types, which are the first real
   candidates for `assertion/`.
3. **Text editing** (~50) — the largest, and the one that will need the most from
   `fixture/`: seven view builders and six render-and-read helpers. Several read
   the private `editing` session and stay.
4. **Images and resources** (5 of 12 movable) — bring `render_resolved` back with
   it, and decide there whether `resident_image` is asking for a public reading of
   the resident image cache or is telling those tests to stay.

Two standing rules for those stages, both learned here: **a helper whose signature
names `UiRuntime` can only serve `tests/`**, and **an entity with no caller is
withdrawn, not parked** — the harness is allowed to be incomplete, but it is not
allowed to be aspirational.

---

# The harness pass, second theme set: frame and style

**Status:** five more themes are migrated. `ui_runtime.rs` went from **11,641
lines to 10,160**, its `#[cfg(test)] mod tests` from **8,886 to 7,405**, and its
`#[test]` count from **231 to 197**. The 34 that left are in six new files under
`crates/application/obscur-runtime/tests/`. The non-test half is **2,755 lines,
exactly as before** — this pass does not touch it either, and the `file_size`
register says so.

Suite **6,229 passed / 0 failed** (unchanged: 34 tests changed address, not
existence), `clippy` **366** (unchanged), `fmt` clean, `xtask check` and
`xtask consumer` both pass.

## What moved, and into which theme

| file | tests | the question the file asks |
|---|---|---|
| `tests/frame_scheduling.rs` | 9 | when does a frame run, and what did a committed transaction buy when it asked for one (§3.2, §3.4, §3.6) |
| `tests/frame_phases.rs` | 4 | which phases a frame actually ran, and why it was allowed to skip the rest (§3.4 p5, §4.3) |
| `tests/style_cascade.rs` | 6 | authored style → computed style → paint, by origin and by state (§4.14) |
| `tests/css_motion.rs` | 6 | transitions and `@keyframes` through the motion orchestration (§4.17, css-animations-1 §3.9) |
| `tests/image_resources.rs` | 8 | how many images did this frame paint, and when did the load happen (§3.3, §3.4, §5.15) |
| `tests/virtual_realization.rs` | 1 | a `<VirtualFor>` marker becomes a viewport-bounded window of real nodes (§4.9) |

Every one keeps its name, its assertions and its intent. What changed is the
arrangement line at the top — `counter_view()` became `CounterPanel::view()`,
`counter_child(&runtime)` became `NodePath::new(&[0]).resolve(&runtime)` — and,
in two places, a rule spelled out as a `Vec<BlockDeclaration>` became the
fixture's own vocabulary (`ButtonElement::filled(ButtonElement::UA_FILL)`).

## What the harness gained

Eight entities, all in folders §2.4's charter already named — no new folder:

| file | entity | what it arranges or reads |
|---|---|---|
| `fixture/counter_panel.rs` | `CounterPanel` | the 100×50 `.counter` element nearly every frame assertion is written against, its sheet, its width-less sheet for animation, and `nested_view` — the one extra child that makes "did the reconcile phase run" observable |
| `fixture/hover_box.rs` | `HoverBox` | the `.box` and the **pair** of hover sheets that make hovering it cost either a relayout (`width`) or only a repaint (`background-color`) |
| `fixture/button_element.rs` | `ButtonElement` | a `button` **type** selector rather than a class, so origin — not specificity — decides the cascade tests; plus the reserved `:disabled` state view and the UA fill both sides name |
| `fixture/image_panel.rs` | `ImagePanel` | the element an image arrives at, by `background-image: url(…)` and by element `src` |
| `fixture/virtual_row_list.rs` | `VirtualRowList` | the `<VirtualFor>` marker over a 24px row template |
| `fixture/resolved_frame.rs` | `render_resolved` | the "render → drain → render" §2.4 calls snapshot normalization — brought back from the withdrawal the last pass parked it in |
| `selector/semantic_selector.rs` | `SemanticSelector` | nodes counted by class, for the one tree whose *shape* is what the test is asking about — likewise un-withdrawn |
| `mock/queued_executor.rs` | `QueuedExecutor` | the blocking port that runs nothing until told, so "the frame did not wait for IO" is observable |

Both entities the last pass built and withdrew for want of a caller are back,
and this is the reason they were parked rather than deleted: the rule is *an
entity with no caller is withdrawn, not parked*, and the corollary is that it
returns the moment the theme that needs it arrives.

`QueuedExecutor` is worth one line of its own, because it is the first fixture
that crosses the boundary in the *other* direction. `BlockingExecutor` is a trait
from `obscur-task`, a foundation crate built once — so unlike anything naming
`UiRuntime`, one `Arc<QueuedExecutor>` satisfies `set_blocking_executor` on both
the library's runtime and the library-under-`--test`'s runtime. The unit tests
that stayed use the harness's copy; there is no second copy.

## The four themes that could not move, and why

This is the more useful half of the report, because each is a *rule* rather than
an accident.

**The seven DPI-variant tests stay** (`a_1x_surface_loads_the_authored_asset…`
through `a_scale_change_within_one_density_keeps_the_resident_image`). They read
the resident decoded image out of the private `resources` field through a
`resident_image` helper, and one of them reads `ResourceBridge::state_of`, which
an earlier pass made `#[cfg(test)]` under this campaign's own rule 4. The last
pass left the decision open — "is `resident_image` asking for a public reading of
the resident image cache, or is it telling those tests to stay?" — and the answer
is that they stay. A *painted* image is publicly readable (`DrawImage` carries
its `SceneImage`), so the assertion could have been rewritten to read the display
list instead; that is a different claim about a different object, and rewriting an
assertion to change a test's address is exactly what this campaign forbids.

**The three `FramePlan` classification tests stay**
(`first_frame_runs_all_phases_even_when_the_reason_is_input`,
`input_frame_skips_reconcile_yet_restyles_and_composes`,
`diagnostics_capture_frame_is_idle_and_reuses_the_retained_scene`). Each is half a
unit test of `FramePlan::for_frame(…)` and half an end-to-end frame, and
`FramePlan` is crate-private. Making it public to move a test is rule 4; splitting
each test in two is rewriting it.

**The three `measure_cache`/`arrange_cache` tests stay** — the incremental-layout
tests that assert *which nodes were re-measured*, not merely that layout ran. They
read two private caches, and the reading has no public name that means the same
thing.

**The two `peek_frame_reason` resource tests stay**
(`an_installed_executor_moves_the_load_off_the_frame`,
`polling_with_nothing_finished_costs_nothing`). Same rule; `peek_frame_reason` is
private and accounts for ten stranded tests across the file.

The shape of all four is one rule: **a test moves when it is written against the
public API, and the API does not move to accommodate a test.** Of the twelve
image tests, five could move; of the seven phase tests, four could. A theme that
comes out in halves is still a theme — the half that moves is read together in
`tests/`, and the half that stays says why, in a comment, where it is.

## What this pass found

- **A duplicated fixture.** `two_node_view` was defined *twice*, identically,
  inside two different tests 90 lines apart (`a_committed_transaction_decides_the_frames_phases`
  and `two_commits_between_frames_both_get_their_phases`) — a local `fn` in a test
  body each time, so nothing pointed out that they were the same view. There is
  now one, `CounterPanel::nested_view`, and the paragraph explaining *why* the
  second view has to be structurally different is on it rather than repeated.
- **A comment separated from the item it describes.** In the test module's import
  block, "The paint commands the device-scale/scroll root-transform tests read
  back, now emitted by `compose_phase.rs`" sat above `use obscur_motion::value::MotionValue;`
  and described `use obscur_scene::command::PaintCommand;` one line below it. This
  is the campaign's recurring defect wearing different clothes: not a missing blank
  line, but an item inserted *between* a comment and its subject. Removing the
  `MotionValue` import (its last user moved) restored the adjacency.
- **A stale parenthetical.** The `AffineTransform` import's comment claimed
  "`Keyframe` is already imported further down, next to the motion tests that use
  it" — a cross-reference to an import that this pass deleted along with those
  tests. Trimmed to describe only the import it is on.
- **A comment that had become true again.** `render_resolved`'s doc in `src/` said
  "the harness spells this too, and for the same reason as `counter_child` it
  cannot be borrowed here". It was false when written — the harness had withdrawn
  `render_resolved` for want of a caller. It is true now, which is the tidier
  outcome than editing it twice.
- **An undocumented helper.** `size(width, height)` sat bare between two
  documented neighbours; it has a doc comment now.

## What the next reader inherits

Three themes are left in `ui_runtime.rs`, and they are the ones the last pass
already named:

1. **Pointer/hover/focus dispatch** (~20 tests) — already on `InteractionPanel`;
   needs `panel_nodes` to become `InteractionPanel::nodes` at the call site.
2. **Command routing and event listeners** (~30) — the five-link chain, the phase
   walk, `stopPropagation`. Two of them read `routing` and `dispatch_command_from`
   and will stay; the `PhaseLog`/`KeySeen` recorders are the first real candidates
   for `assertion/`.
3. **Text editing** (~50) — the largest, and the one with the most private reads;
   expect it to come out in halves like the image theme did.

Plus three smaller residents that belong to no theme yet and may never: the
telemetry sink tests (7), the locale-wiring tests (6), and the scattered runtime
basics (`new_starts_idle_with_identity_and_viewport`, `render_now_is_deterministic`,
`debug_reports_identity_fields`).

The standing rules are unchanged, with one added here: **a fixture whose
signature names only foundation-crate types can serve both sides of the
`--test` boundary** — which is why `QueuedExecutor` has one home and
`counter_child` still has two.

---

# The harness pass, third theme set: the input side

**Status:** five more themes are migrated. `ui_runtime.rs` went from **10,160
lines to 8,737**, its `#[cfg(test)] mod tests` from **7,405 to 5,982**, and its
`#[test]` count from **197 to 152**. The 45 that left are in five new files under
`crates/application/obscur-runtime/tests/`. The non-test half is **2,755 lines,
exactly as before** — this pass does not touch it either, and the `file_size`
register says so.

Suite **6,229 passed / 0 failed** (unchanged: 45 tests changed address, not
existence), `clippy` **366** (unchanged), `fmt` clean, `xtask check` and
`xtask consumer` both pass.

## What moved, and into which theme

The brief named one region — pointer dispatch, hover and focus, keyboard and
shortcuts, command dispatch and §2.6's chain. It came out as five themes, not
one, because the questions are genuinely different and a file that answered all
of them would have been the test module again under another name.

| file | tests | the question the file asks |
|---|---|---|
| `tests/pointer_dispatch.rs` | 10 | where a pointer lands, and what that costs the tree: hit-test → hover → focus → pseudo-class → paint (§2.3, §4.14) |
| `tests/click_commands.rs` | 8 | a click resolves a command, and the event says which node it was *attached to* (§2.3, §2.6) |
| `tests/command_routing.rs` | 4 | §2.6's five links, tried in order, first claim stopping the chain |
| `tests/event_listeners.rs` | 12 | §2.3's event model driven end to end: capture, target, bubble, default action — and the payload a handler is given |
| `tests/keyboard_commands.rs` | 11 | the keyboard as a command *surface*: routing to focus, shortcuts, the §6.7 unified `Command`, Enter/Space activation |

Every one keeps its name, its assertions and its intent. What changed is the
arrangement line at the top — `UiRuntime::new(runtime_id(), viewport())` became
`TestFixture::runtime()`, `panel_nodes(&runtime)` became
`InteractionPanel::nodes(&runtime)`, `panel_single_child(&runtime)` became
`NodePath::new(&[0]).resolve(&runtime)`, and one bespoke sheet spelled as a
`Vec<BlockDeclaration>` became `ClassRule::new("a").sized(100.0, 50.0).build()`.

## Why the five are five

`pointer_dispatch` and `click_commands` look like one theme and are not. The
first asks what a *contact* does to the tree — which node it resolves to, which
pseudo-classes light up, which of them survive the release, and (twice) whether
the resulting paint changed, because `.b:hover` and `.a:focus` are the only rules
in `InteractionPanel::sheet` that paint anything, so a fill absent from the cold
frame and present in the next is proof the interaction reached the cascade. The
second asks what a *pair* of contacts on one node resolves to, which is a
different object: a click is not a pointer event, and the binding it finds may
live on an ancestor. That is why `target` and `current_target` are two values,
and eight tests exist to keep them apart.

`command_routing` and `event_listeners` divide the same way. The chain is the
**outward** leg — five links, each consulted only when every inner one declined —
and the listener walk is the **whole** propagation, capture leg included, with
the chain as its default action. `stop_propagation` ending §2.6's dispatch is
precisely the fact that they are one mechanism, and it is asserted in the file
about the walk, where the reader can see the capture listener that caused it.

`keyboard_commands` is the surface argument: a key press and a click are two
doors onto one command model. Its centre is the §6.7 test where **one**
`CommandState` is disabled and *both* the shortcut and the click go inert, then
both come back — which is a claim about neither surface alone.

## What the harness gained

Two entities, both in folders §2.4's charter already named — no new folder:

| file | entity | what it arranges or reads |
|---|---|---|
| `assertion/handler_calls.rs` | `counting_handler` | a counter and a handler that bumps it: the handler goes to the runtime, the counter stays with the test |
| `fixture/interaction_panel.rs` | `InteractionPanel::ancestor_bound_view` | the same panel with the *container* bound and `a` left unbound, so a click on `a` must bubble |

`counting_handler` is the pass's one deletion from `src/`. Its signature names
`Rc`, `Cell` and `FnMut` and nothing else — no crate that this target builds
twice — so unlike `panel_nodes` it can serve **both** sides of the `--test`
boundary, and it does: the two routing tests that stayed in `ui_runtime.rs` now
import it from the harness, and so do three of the five new files. This is the
standing rule from the last pass (`QueuedExecutor`'s rule) applied for the first
time to something that *already existed* in `src/` rather than to something built
new.

`ancestor_bound_view` earned its place by having two callers in two different
files: `click_commands.rs` needs the bubbling arrangement to prove `target` and
`current_target` differ, and `keyboard_commands.rs` needs the same view to ask
whether an Enter activation fires the focused node's own binding. Adding it as a
third view builder on `InteractionPanel` rather than as a second panel fixture
keeps the geometry documented in one place — the 100×100 container over two
stacked 100×50 children that every y-coordinate in five files depends on.

Nothing else was added. `two_row_view`, `two_rows`, `click_at`, `checkbox_view`,
`key_a`, `bound_view` and `focused_on_a` are each used by exactly one of the new
files and stayed there as private local functions, which §3.1 allows and the
"withdrawn, not parked" rule requires: a fixture with one caller does not need a
crate to live in. Two of them are now one line over a harness call
(`bound_view` is `InteractionPanel::ancestor_bound_view(CMD_PANEL, CMD_B)`),
which keeps every following line of the tests byte-identical — the diff should
read as "the fixture changed shape", never as "an assertion changed".

## The four that could not move, and why

**The two §2.6 chain tests that read past the API stay.**
`the_five_links_are_tried_in_the_specs_order_and_the_first_claim_stops_the_chain`
takes the window link away mid-test by calling `routing.remove_command` on a
private field; `a_focus_scope_link_appears_only_for_its_own_members` dispatches
from a node that is *not* the focused one through the private
`dispatch_command_from`. Both are the chain read from the inside, which is what a
unit test is for, and neither could move without widening the API to accommodate
a test. Four of the six moved; the theme came out in halves, like the image and
phase themes before it, and the half that stayed says why in a comment where it
is.

**The two `focus_accepts_text` tests stay**, and for a different reason worth
recording because it is not the usual one. They use only the public API and could
have moved. But `focus_accepts_text_only_while_an_editable_holds_focus` is
written against `input_reactive_view`, the text theme's fixture, and its partner
`a_focused_button_wants_no_text_entry` opens with "The other half" — the pair is
a documented argument that "something is focused" is not the test. Splitting it
across two files to satisfy a theme boundary would cost more than it buys, so
both belong to whichever stage migrates text editing.

## What this pass found

- **A dead helper the migration exposed.** `panel_single_child` had exactly one
  caller — the disabled-control test — and moving that test left it with none. It
  is deleted rather than kept "for symmetry" with `panel_nodes`; the harness's
  `NodePath::new(&[0]).resolve(…)` says the same thing at the one call site that
  still wanted it.
- **A section header that outlived its section.** `// ---- Phase 3 step 2: input
  dispatch (hit-test → hover/focus → pseudo) ----` labelled a block that, after
  this pass, contains no tests at all — only the panel view, its sheet and its
  node selectors, which the device-scale, content-offset, scrolling and editing
  sections below all drive. It now says what it actually is, and names the four
  files its tests went to.
- **A fixture assertion that named a private field for no reason.**
  `focused_on_a` asserted its own precondition with `runtime.focused` where
  `runtime.focused()` returns exactly that field. The copy that moved to
  `tests/command_routing.rs` uses the accessor; the copy that stays is untouched,
  because the two tests it still serves reach past the API anyway.
- **No misplaced doc comment.** The class of defect this campaign found eleven
  times does not appear in this region either — the same reason as last pass:
  test-module helpers use ordinary `///` comments and are short.

## What the next reader inherits

Two themes and three residents:

1. **Text editing** (**56** of the remaining 152, across thirteen labelled
   sections from `EVENT_INPUT` to IME to the mouse caret) — the largest by far,
   and the one with the most private reads (`editing`, `edit_is_compose_only`).
   Expect it to come out in halves like the image, phase and routing themes did,
   and expect it to need the most from `fixture/`: seven view builders and six
   render-and-read helpers, most of which name `UiRuntime` and so can only serve
   `tests/`. The two `focus_accepts_text` tests above belong to it.
2. **Components and error boundaries** (**35** — 14 in `runtime-driven
   components`, 21 in `§1.7 boundaries, §1.6 diagnostics`) — two nested test
   modules carrying their own component factories, props decoders and diagnostic
   sinks. These are the first real candidates for `mock/` and for `host/`, which
   has held nothing but the `PluginTestHost` stub since the harness was written.

Plus the residents that belong to no theme and may never: scroll-into-view (11),
the device-scale and content-offset surface tests (11), the invalidation and
text-shaper tests under *Audit S1* (8), the telemetry sink tests (7), the
locale-wiring tests (6), the reactive re-render loop (5), the accessibility
publication tests (4), CSS motion (4), and the scattered runtime basics.

The standing rules are unchanged. This pass adds a corollary to the last one's:
**a helper in `src/` whose signature crosses the boundary should be moved to the
harness, not copied** — `counting_handler` had no business being defined twice,
and the moment a second target needed it, the duplicate was the alternative.

---

# The harness pass, fourth theme set: everything else

**Status:** the sweep. Fourteen more files, **120 tests**, and the module is
down to the tests that genuinely cannot leave. `ui_runtime.rs` went from
**8,737 lines to 4,381**, its `#[cfg(test)] mod tests` from **5,982 to 1,626**,
and its `#[test]` count from **152 to 32**. The non-test half is **2,755 lines,
exactly as before** — the fourth pass in a row that does not touch it.

Suite **6,229 passed / 0 failed** (unchanged: 120 tests changed address, not
existence), `clippy` **366 → 363**, `fmt` clean, `xtask check` and
`xtask consumer` both pass.

## What moved, and into which theme

The brief named five regions — text editing and IME, scrolling, accessibility,
DevTools inspection, diagnostics and telemetry — and then said *sweep the rest*.
DevTools turned out to have nothing here: `TreeInspector` took its tests with it
when it was extracted, and `ui_runtime.rs` only delegates to it. The other four
came out as fourteen files, because the sweep and the named regions are the same
work once the question is "what is this file *about*".

| file | tests | the question the file asks |
|---|---|---|
| `tests/text_input_events.rs` | 12 | what a field *tells the app*: `input` per keystroke, `change` once on commit (WHATWG HTML §4.10.5.5) |
| `tests/text_editing_session.rs` | 12 | what the session holds while a field has focus, and who may move it — keyboard, controlled rewrite, `readonly` |
| `tests/text_multiline.rs` | 9 | text that occupies more than one line: `Enter`, the line-aware caret, vertical caret scroll, measure↔compose |
| `tests/text_ime.rs` | 5 | §5.12 composition: the text a user can see but has not yet typed |
| `tests/text_mouse_caret.rs` | 10 | the pointer as a caret — where a press lands, what a drag selects, what a *run* of presses means (§5.11) |
| `tests/text_shaping.rs` | 11 | what reaches the shaping port and how often, including the language it carries (§7.2) |
| `tests/scroll_containers.rs` | 11 | where a node has to be for the user to see it (CSS Overflow 3, §6.15, §4.8) |
| `tests/surface_mapping.rs` | 11 | the render transform and the pointer mapping as exact inverses |
| `tests/semantic_tree.rs` | 5 | §7.1: what the runtime derives from the retained tree, and how it reaches the platform |
| `tests/phase_telemetry.rs` | 7 | §7.9: what a frame costs, attributed to the frame that spent it |
| `tests/component_lifecycle.rs` | 9 | §2.1/§2.14: mounting, props-driven re-render, the reconcile that must not eat the output, unmount |
| `tests/error_boundaries.rs` | 8 | §1.7 containment and §1.6 reporting, with the failure held fixed and only the policy varying |
| `tests/reactive_rendering.rs` | 5 | the held view builder: click → app state → rebuild → reconcile |
| `tests/runtime_identity.rs` | 5 | what a `UiRuntime` is before, during and after one frame |

## Why some themes split and others merged

**Text came out as five**, not one, and the seam is what each file *reads*.
`text_input_events` reads the app's own handlers; `text_editing_session` reads
`editing_text`/`editing_selection`; `text_multiline` reads painted caret
geometry against a line index; `text_ime` reads the buffer and the display
*separately*, because keeping those apart is the whole subject; `text_mouse_caret`
reads a byte offset back from a coordinate. A single file would have been the
test module again under another name.

**Shaping and locale merged.** They arrived as two regions — the shaper port and
`set_locale` — and they are one seam: both install a stub through a public
`set_…` and read the result out of the composed display list, and the two cache
assertions that give `set_locale` its teeth (`switching_locale_reshapes…`,
`a_locale_switch_remeasures…`) are shaping-cache claims wearing a locale hat.
Eleven tests, 442 lines, one question.

**Device scale and content offset merged**, for the reason the extraction
already found: they are `SurfaceGeometry`'s single invariant seen twice. The
file asserts each of the four claims against the other, so a change to one that
forgets the other fails there.

**Components split from boundaries.** The lifecycle file asks what the runtime
does to an instance; the boundary file asks what it *reports* when a render
fails. They share a fixture and nothing else, and the boundary file's whole
design — one failure held fixed, only the policy varying — only reads that way
once the lifecycle cases are somewhere else.

## What the harness gained

Seven entities, in the folders §2.4's charter already named. No new folder.

| file | entity | what it is |
|---|---|---|
| `fixture/key_event.rs` | `char_press`, `named_press` | the keyboard's `mouse` — foundation types only, so one definition serves both sides of the `--test` line |
| `fixture/editable_panel.rs` | `EditablePanel` | one field binding `on-input`, in `InteractionPanel`'s geometry, plus the placeholder overlay and a focused-and-seeded runtime |
| `fixture/single_line_field.rs` | `SingleLineField` | the two overflow regimes of a single-line field: 100 px `nowrap`+`hidden`, and 40 px wrapping with an exact line height |
| `fixture/text_area.rs` | `TextArea` | the multi-line control, with and without a scrollport |
| `host/counter_component.rs` | `CounterComponent`, `CounterProps` | a component whose rendered text answers "did the props decode", "did render run" and "did the instance survive" at once |
| `host/guarded_component.rs` | `GuardedComponent` | the component that fails on purpose, taking the *same* props so the failure is fixed and only the policy varies |
| `host/enclosing_component.rs` | `EnclosingComponent` | a boundary above the one that failed |
| `selector/semantic_selector.rs` | `first_text`, `child_text` | the two readings a rebuilt view and a mounted component are asserted through |

`host/` had held nothing but the `PluginTestHost` stub since the harness was
written. The three components landed there rather than in `fixture/` because
§2.4's charter distinguishes a *fixture* (a value a test builds) from a *host*
(code the runtime mounts and drives), and a `ComponentFactory` the runtime calls
back into is the second thing.

The one naming collision worth recording: `obscur-runtime` already owns a
domain entity called `EditableField` (`runtime/editable_field.rs`), which
`use super::*` brings into the test module's scope. The fixture is
**`EditablePanel`**, and it is named after the panel it renders rather than
after the field inside it, so the two never have to be told apart by their path.

## The 32 that stayed, and why

Six reasons, and every one of them is "a claim about something the public API
does not expose":

| tests | what they read |
|---|---|
| 10 | `peek_frame_reason` — five signal/`request_update` cases, a boundary retry, two compose-only edits, two async loads. "It asked for a frame" and "it did not" are the assertions, and the pending reason is private |
| 7 | the `resources` field, through `resident_image` — the DPI-variant tests read the decoded image's pixel size, which the hub exposes to `#[cfg(test)]` only |
| 5 | the `editing` field, through `scroll_left_of` — a field's horizontal scroll offset has no accessor |
| 3 | `FramePlan::for_frame(…)` — the classification read from the inside; `FramePlan` is crate-private |
| 3 | `measure_cache` / `arrange_cache` — "only the hovered node was re-measured" is a statement about a cache, not about output |
| 2 | `routing.remove_command` and `dispatch_command_from` — §2.6's chain taken apart link by link |
| 1 | the `focused` field — a focus *push* reports moving from the same field it records having published, and only one of those two writes is public |
| 1 | `MULTI_CLICK_WINDOW_MICROS` — `pub(crate)`, and the test exists precisely to prove the window is checked |

The last one is the pass's one genuinely new finding. Nine of the ten
mouse-caret tests moved; `a_press_after_the_multi_click_window_is_a_fresh_single_click`
could not, because it is the only test in the workspace that *names* the window
constant — and hard-coding 500 ms to relocate it would have deleted the thing it
guards. Its doc comment already explains why the window is the condition a
caller can most easily make vacuous; it now also explains why it sits alone.

## What this pass found

- **A false positive that was worth chasing.** A first scan flagged
  `a_frame_records_every_phase_span_and_counter_into_the_sink` as reading the
  private `boxes` field. It does not: it asserts on the *string* `"layout.boxes"`,
  a counter label. The test moved.
- **A crate-private *type*, which the same scan could not see.** The three
  invalidation tests name `FramePlan`, not a field or a function, so a scan for
  `self.<field>` and `<private_fn>(…)` missed them entirely. They were caught by
  the compiler, and by `tests/frame_phases.doc/_module.md`, which had already
  written down exactly this reason a pass earlier. The lesson for the next
  reader: the scan finds private *access*, and the doc comments find private
  *dependency*.
- **Three helpers with no callers left.** `counter_child`, `semantic_with_role`
  and `background` each served only tests that left. They are deleted rather than
  kept "in case"; `counter_child`'s doc even argued for its own existence, which
  is exactly the comment that goes stale silently.
- **Four fixtures that were defined twice.** `scroll_field_view`/`_sheet` and
  `textarea_view`/`_sheet` now delegate to `SingleLineField` and `TextArea`: the
  geometry every caret assertion depends on is stated once, and the two local
  wrappers exist only because their callers read better without the type prefix.
- **No misplaced doc comment.** The class of defect this campaign found eleven
  times does not appear here either.

## What the next reader inherits

Nothing, in this file. Every test that can leave `ui_runtime.rs` has left it;
the 32 that remain are enumerated above with the private item each one reads,
and moving any of them would mean widening the public API to serve a test, which
is the one thing this campaign has refused throughout.

The standing rules are unchanged. This pass adds one: **a scan for private
access finds fields and functions; a crate-private *type* is invisible to it,
and the compiler is the only reliable check.** Run the target before believing a
list.

---

# Closing the books on the harness arc

**Status:** verification only. **Nothing was migrated by this pass**, and no
production or test code changed. Its job was to re-measure the four harness
passes from the tree rather than from their reports, and to correct the three
documents that were still describing a workspace that no longer exists.

## What was re-measured, and what it came to

Every number below was taken from the tree, not from a report.

| claim | measured |
|---|---|
| `ui_runtime.rs` total lines | **4,381** |
| …of which non-test | **2,755** |
| …of which `#[cfg(test)] mod tests` | **1,626** |
| `#[test]` still in `ui_runtime.rs` | **32** |
| `#[test]` in `crates/application/obscur-runtime/tests/` | **233**, over 26 files |
| 32 + 233 | **265** — the count the campaign started with, to the test |
| suite | **6,229 passed / 0 failed** |
| `clippy --workspace --all-targets` | **363** warnings |
| `fmt`, `xtask check`, `xtask consumer` | clean / pass / pass |

The 265 reconciles exactly. That is the strongest statement this pass can make
and the only one worth making: **no test was gained, lost, merged or split**;
233 of them changed address and 32 did not.

The 32 were re-derived independently, by brace-matching each `#[test]` body and
searching it for the private items the campaign named. **All 32 hit at least one;
none came back clean.** So the standing claim — that every remaining test reads
something the public API does not expose, and that moving it would mean widening
the API to serve a test — holds under a check that did not consult the list it
was verifying. Two corrections to the arithmetic in the earlier reports, neither
changing a verdict: `swapping_the_shaper_drops_the_retained_measures` and
`a_focus_scope_link_appears_only_for_its_own_members` were each credited with
more private reads than they make, because a first scan bounded a test's body at
the *next* `#[test]` and swept up the helpers in between.

## The 26 files, and the bar

The largest test file is `tests/touch_gestures.rs` at **971 lines** — under the
1,000-line bar, but only by 29, and it is the one file in `tests/` worth
watching. The next largest is `text_input_events.rs` at 500. `xtask`'s
`file_size` rule counts a file with no `#[cfg(test)] mod tests` in full, so an
integration test file is measured whole: `touch_gestures.rs` is 29 lines from
being a violation, and the next touch-gesture test to be written should open a
second file rather than extend that one.

## What the ledger now says, and why the entry stays

`tools/xtask/src/rule/file_size.rs` **keeps** its one `KNOWN` entry. The rule
requires deleting an entry whose file has come under the cap, and this one has
not: `ui_runtime.rs` is **2,755 lines of non-test code**, and the rule counts
nothing else. A whole-workspace re-measure confirms it is still the **only** file
over the bar — the next largest non-test count in the workspace is
`obscur-runtime/src/runtime/resource_bridge.rs` at 723, and outside `src/` the
largest anything is `touch_gestures.rs`'s 971.

This is the fact the four harness passes leave standing, and it deserves to be
said plainly rather than buried in a percentage: **the harness arc did not move
this file toward the bar at all.** It could not. It emptied the half the rule
does not count — 9,978 test lines down to 1,626, a 78%-to-37% shift in what the
file *is* — and left the half the rule does count byte-for-byte identical, four
times in a row, deliberately. What retires the entry is still what the sixth
pass named: the 481 lines of inline commentary and the 1,335 lines of private
methods. The harness arc's contribution to that work is that the file is now
legible enough to attempt it.

## The three documents that were lying

- **`tools/xtask/src/rule/file_size.rs`** recorded passes seven through nine and
  stopped. It now records the tenth and states the arc closed, with the six
  reasons the 32 stay named in the entry itself, so the next reader of the
  ledger does not have to come here to learn why the number did not move.
- **`docs/api/12-reality.md`** listed `obscur-testkit` as *"unreachable by
  construction — depended on by nothing, and not re-exported"*. That was already
  corrected by the first harness pass; this pass sharpened it to name the crate's
  scale (36 files, ~1,900 lines, 131 public items) and its **first and only
  consumer** — `obscur-runtime`'s own `tests/` suite — and kept the old sentence
  visible beside the correction, because the row's value is the contrast.
- **`docs/conformance-audit.md`** still said it in two places, in a document
  whose whole convention is to strike a finding through when it closes. Both are
  struck now, tagged **7-E**, with the two things the revival did *not* close
  stated rather than glossed: `PluginTestHost` is still a bare stub, and testkit
  has **no consumer outside `obscur-runtime`** — so §7.5's promise that
  *downstream* crates test through the harness is still unmet.

## What this pass found

- **A public surface wider than its callers, in the harness.** Four public items
  in `obscur-testkit` have no caller outside their own file:
  `LongPressPanel::CMD_CLICK`, `LongPressPanel::CMD_HOLD`,
  `ScrollingList::CMD_ROW` and `ScrollingList::view_with_click`. They are not
  dead — each is used by the fixture that declares it — but `view_with_click` is
  *parameterized* by a command whose only argument is the constant next to it,
  which is the shape rule 4 was written against. It is recorded rather than
  narrowed: the first harness pass established, and the owner accepted, that a
  fixture keeps the shape it was built with rather than this month's callers, and
  a verification pass is the wrong place to start relitigating that. Next reader:
  if `view_with_click` still has one caller, make it private.
- **The harness's own reach is the real open item, not its contents.** 131 public
  items, one consumer crate. §2.4's charter and §7.5's promise are both about
  *downstream* crates testing through it; today the harness serves the crate it
  was extracted from and nothing else. That is a smaller claim than "revived" and
  it is the honest one.
- **No misplaced doc comment.** The defect this campaign has found eleven times
  does not appear anywhere in the four passes' output — checked across all 26
  `tests/` files and all 36 testkit sources.

## What the next reader inherits

The test module is finished. Whoever picks this file up next is working on the
**2,755 lines above it**, with the whole of `docs/runtime-decomposition.md`
behind them and the `KNOWN` entry naming the two remaining candidates in order.

One rule the arc adds to the standing set, and it is about verification rather
than extraction: **a report is not a measurement.** Every number in this chapter
was re-derived from the tree, and two of them came back different from what was
reported — not wrongly, but derived by a method that could not have caught what
it missed. Bound a test's body by matching braces, not by finding the next
attribute.

---

# The facade pass: child modules, and the delegate that needs no delegate

**Status:** the mechanism is established and its first load has landed.
`ui_runtime.rs` went from **2,755 to 2,184 lines of non-test code** (4,381 →
3,822 total); its `impl UiRuntime` from **2,573 lines over 179 methods to 1,990
over 82**, and the public half of that impl from **139 methods to 42**. Fifteen
child modules were created, **732 non-test lines** between them, the largest 127
and the smallest 22 — every one of them clear of the bar by a factor of eight.

Suite **6,229 passed / 0 failed** (unchanged — no test was moved, weakened or
deleted), `clippy` **363** (unchanged, and none of the fifteen new files
contributes one), `fmt` clean, `xtask check` and `xtask consumer` both pass,
`cargo doc` renders all 97 moved `#[doc = include_str!]` attributes on the same
`UiRuntime` page they were on before.

## The route the facet proposal missed

*The audit pass* concluded that reaching 1,000 meant splitting `UiRuntime`'s
public surface into thirteen `XxxFacet<'a>(&'a mut UiRuntime)` types, priced the
change at **2,842 call sites across 253 files**, and handed the decision to the
owner because a refactor cannot spend that on its own authority. It also ruled
out the cheap version: leaving forwarding methods behind puts ~19 lines back per
method and the file does not move.

Both halves of that arithmetic rest on the same assumption — that a method's body
can only leave this file inside *another type*. It cannot, because
`UiRuntime`'s 47 fields are private and a sibling module cannot see them; moving
a body to `runtime/whatever.rs` would mean widening the struct to `pub(crate)`.

A **child** module is the exception Rust already grants. A private field is
visible to its defining module *and its descendants*, so
`ui_runtime/resources.rs` reads `self.resources` exactly as `ui_runtime.rs` does,
with the encapsulation unchanged. And because a child module can hold
`impl UiRuntime { … }` directly, a moved `pub fn` **is still `UiRuntime::name`**
— same path, same signature, same doc, same call sites. There is no facet to
reach through and no delegate to leave behind. The price the facet proposal
priced at 2,842 call sites is zero.

That reading of §3.1 is the one this pass acts on: the rule admits one main
entity, its `impl`, and helpers with no independent semantics. It says nothing
about the `impl` living in one file, and §3.16's question — *how many main
entities does this file export?* — is answered "one" by fifteen files that
declare no type at all.

## What moved, and into which module

The load was chosen for being judgement-free: the **97 public methods that were
already one-line delegates**, which is pure relocation. Each moved whole, with
its `#[must_use]`, its doc attribute, and its `.md` relocated into the new file's
own `.doc/` directory.

| module | methods | non-test lines | the subject |
| --- | --- | --- | --- |
| `command_routing.rs` | 19 | 127 | §2.6/§6.7: the registry, the router, the shortcuts, and every level of the routing chain |
| `surface.rs` | 10 | 64 | the viewport, its mapping to host pixels, and the §6.16 environment insets that bite into it |
| `resources.rs` | 9 | 65 | the four ports a host installs, and the §5.19 cache's budget, policy and counts |
| `styling.rs` | 8 | 56 | what goes into the cascade — sheets, pseudo states, locale — and the one reader of what comes out |
| `pending_work.rs` | 7 | 50 | what is outstanding: frames, queued input, background units, and §3.6's starvation signal |
| `diagnostics.rs` | 7 | 49 | the two sinks, the frame budget, and the §7.6 inspection readers |
| `frame_output.rs` | 7 | 54 | what a frame leaves behind to be read, and the identity it is attributed to |
| `event_listeners.rs` | 6 | 50 | §6.5 capture/bubble listeners, plus `input` and `change` |
| `pointer.rs` | 5 | 39 | releasing a capture, and the §6.9 drop targets, session and outcome |
| `virtual_lists.rs` | 4 | 42 | §4.9 registration and the realized window |
| `view_source.rs` | 4 | 34 | the two sources of a desired view: the root builder and the component registry |
| `accessibility.rs` | 3 | 28 | §7.1's two projections and whether a bridge is installed |
| `motion.rs` | 3 | 25 | §4.17's CSS rules and the two readings of how many timelines run |
| `interaction_state.rs` | 3 | 27 | `hover`, `focused` and `modifiers` — the three fields the state pass refused, and their one reader each |
| `text_editing.rs` | 2 | 22 | the session's two readers, the pair that only reads |

The grouping is by **subject**, following the entity each method delegates to,
and two of the rows are worth their reasoning:

- **Commands and events are two files, not one.** They install onto the same
  `HandlerRegistry`, which is an implementation fact. The vocabularies differ: a
  command is a named action that travels a routing chain, can be enabled and
  disabled, and can be bound to a shortcut; a listener is *told* something
  happened to a node and cannot refuse it. `on_text_input` and `on_change` are
  listeners by that test, not commands.
- **The focus scopes went with the commands and the focused node did not.** A
  scope is a link in the routing chain, opened and closed on the registry; the
  focus itself is a runtime field the chain merely starts from, and it belongs
  with the two other fields nothing could extract.

## What did not move, and why

The 42 public methods that still carry a body, and all 40 private ones, are for
the stages after this one — the point of a first pass is to prove the mechanism
under a load where nothing can be got wrong. Two others stayed for reasons that
will not change:

- `virtual_index_at` and `scroll_virtual_list` read the retained boxes and the
  scroll offsets as well as the list geometry, so they are not delegates; the
  section banner above them now says where their registration half went.
- The `#[cfg(test)] mod tests` stayed whole. Its 32 tests each read a private
  item, and a child module does not change that.

## What this pass found

- **No misplaced doc comment.** All 97 attributes sat on the item they described,
  which keeps the campaign's count of that defect at eleven.
- **Six `.md` files carried a `_2` suffix for a reason that no longer exists.**
  `locale_2.md`, `environment_2.md`, `focused_2.md`, `tree_2.md`, `scene_2.md`
  and `id_2.md` were named around a collision with a *field* doc of the same name
  in the shared `ui_runtime.doc/`. Each moved into a directory where no such
  collision exists and took the name of the item it documents. Contents
  untouched.
- **Twenty-one intra-doc links resolved only by accident of scope.** A shortcut
  link like ``[`UiEvent`]`` resolved because `ui_runtime.rs` happened to import
  the type for its own code; moved into a module that does not, it breaks. Adding
  the import back is not the fix — `rustc` then reports it unused, because the
  lint does not know about doc links. Each is now a markdown link-reference
  definition at the foot of its own `.md` (``[`UiEvent`]:
  obscur_interaction::event::UiEvent``), which renders identically and resolves
  from anywhere. Worth knowing before the next pass moves a body with links in
  it: `cargo doc` is part of this campaign's green, not an afterthought.
- **One pre-existing warning, deliberately left.** `set_root_view`'s doc links to
  `Self::run_frame`, which is private; `poll_resource_tasks` has had the same
  warning in `ui_runtime.rs` all along. It travelled with its method rather than
  being fixed in passing.

## What the next reader inherits

The mechanism, and 1,990 lines of `impl` to apply it to. The arithmetic that
matters: 82 methods remain, and moving all of them leaves the struct, its 47
field docs, the type's doc block, the imports and `Debug` — under 300 lines.
**The 13-facet proposal, and the 2,842 call sites it costs, is no longer the
route to the bar.** It should not be revived without a reason that survives the
observation above.

Two cautions for the bodies, which are not judgement-free the way this load was:

1. `handle_pointer` (250 lines of eleven labelled steps) and `run_frame` (294 of
   phase sequencing over five phases that are already their own entities) should
   decompose into named steps before they move, not be relocated wholesale.
2. Six of the 40 private methods are themselves one-line delegates
   (`resolve_element_vectors`, `peek_frame_reason`, `dispatch_command_from`,
   `invalidate_dpi_variants`, `to_content_point`, `editing_view`) and can ride
   along with whichever subject calls them.

---

# The frame pass: the pipeline moves out, and one question comes out of it

**Status:** the frame and lifecycle surface has left the facade. `ui_runtime.rs`
went from **2,184 to 1,542 lines of non-test code** (3,822 → 3,186 total); its
`impl UiRuntime` from **1,990 lines over 82 methods to 1,346 over 61**. Seven
child modules were created, **751 non-test lines** between them, the largest 315
and the smallest 40 — every one clear of the 1,000-line bar.

Suite **6,229 passed / 0 failed** (unchanged — no test was moved, weakened or
deleted), `clippy` **363** (unchanged, and none of the seven new files
contributes one), `fmt` clean, `xtask check` and `xtask consumer` both pass, and
`cargo doc -p obscur-runtime` emits **the same 56 warnings as before, line for
line** — which is the check that matters here, because moving a body moves the
scope its doc links resolved in.

`ui_runtime.rs` is **still over the bar at 1,542**, and deliberately so: this
stage owns the frame, and `handle_pointer` (254 lines), the text-editing cluster
(~500) and `handle_key`/`handle_ime` (156) are the stages after it. The
arithmetic is in **What the next reader inherits**.

## What moved, and into which module

| module | methods | non-test lines | the subject |
| --- | --- | --- | --- |
| `frame_pipeline.rs` | 3 | 315 | the frame itself: the phase sequence, and the two readings it takes of runtime state on the way through |
| `frame_entry.rs` | 3 | 90 | the three ways a frame is run — scheduled, scheduled-reactive, unconditional |
| `construction.rs` | 1 | 88 | what each of the 47 fields *begins* as |
| `resource_resolution.rs` | 6 | 87 | the resource work a **frame** does: landing, and the three per-frame resolutions |
| `frame_requests.rs` | 6 | 83 | the scheduler's ingress — everything that says there is work, including §3.6's background band |
| `frame_classification.rs` | 1 | 48 | the one question the pipeline asks before it starts |
| `queued_input.rs` | 2 | 40 | the two ends of the `InputQueue`: what fills it, what drains it |

Two of the groupings are worth their reasoning:

- **The frame's resource work is not the resource surface.** `resources.rs`'s own
  module doc, written by the previous pass, already said the deferred-load drain,
  the visibility marking and the three resolutions "stay with the frame, because
  they read the tree, the boxes and the surface as well". That argument is
  inherited rather than re-litigated: `resource_resolution.rs` is those six, and
  `resources.rs` keeps the ports and the cache settings.
- **Background work is in `frame_requests.rs`, not a file of its own.** §3.6 makes
  it a *band of the same scheduler* rather than a second queue: `request_background`
  signals the scheduler exactly as `request_frame` does, and every frame drained
  afterwards counts as a preemption of it. `service_background_unit` is the other
  half of that one piece of bookkeeping, called from the frame entry points before
  a frame is pulled.

## The one thing that came out of `run_frame`

The previous pass's caution was that `run_frame` (294 lines) "should decompose
into named steps before it moves, not be relocated wholesale". It was read for
exactly that, and **one** step came out; the rest is stated to be sequencing and
kept whole. The reasoning, so that it is not re-opened as an oversight:

- **`plan_for_frame` came out** (`frame_classification.rs`). It has its own
  vocabulary and no phase in it: four independent sources — the frame's
  `FrameReason`, a plan staged by `apply_transaction`, a plan the dispatch phase
  folded in, and a resource result that landed this frame — and a precedence
  between them where every rule guards a silent failure. Its interface is four
  scalars in and one `FramePlan` out; its internals are those four rules. It is a
  file of its own for the same reason `LayoutClassifier` is one: refining a plan
  against what the styles actually changed is already an entity, so *choosing* the
  plan is its sibling — the coarse half of one two-step classification (§3.4
  phase 5).
- **The phases did not.** Reconcile, style, layout and compose are already
  `ReconcilePhase`, `StylePhase`, `ArrangePhase` and `ComposePhase`, each in its
  own file. What is left around them is not a hidden algorithm; it is the
  **ordering**, and the ordering is the thing itself: dispatch before the plan is
  chosen, off-thread results committed before the plan is chosen, images resolved
  before layout rather than at paint, the controlled-input reseed after reconcile
  and before measure, accessibility published after both reconcile and dispatch
  have settled. A `style_step` extracted from it would take seven arguments and
  return four values, and its body would be the `StylePhase` constructor it
  already is. That is plumbing bought at the price of the comments that are the
  only record of why each constraint holds.

## The mechanism, one step further than the facade pass took it

The facade pass moved 97 methods that were already one-line delegates. This one
moves **bodies**, and bodies call each other — which surfaced the half of Rust's
privacy rule the previous pass never had to touch. A private item is visible in
its defining module *and its descendants*, so a child module reads
`UiRuntime`'s private fields and calls the parent's private methods for free —
but the **parent cannot call a private method defined in a child**, and neither
can a sibling child.

Nine moved methods have a caller outside their new file (`run_frame` itself,
`dispatch_queued_input`, `commit_resource_results`, the three image resolutions,
`mark_image_visibility`, `service_background_unit`, `peek_frame_reason`), as does
the newly extracted `plan_for_frame`, and each of the ten is now
**`pub(super)`** — which from inside `ui_runtime::frame_pipeline`
means `pub(in crate::runtime::ui_runtime)`: visible in `ui_runtime` and its
descendants, which is *precisely* the scope a private method in `ui_runtime.rs`
had. **No item is visible anywhere it was not visible before**, and no field was
widened. `peek_frame_reason` is the proof that the scope is exact: it is
`#[cfg(test)]`, its only callers are the fifteen assertions in the parent file's
own test module, and they compile unchanged.

## What this pass found

- **A misplaced comment of the campaign's recurring family — in `//` form.** Two
  contiguous comment paragraphs sat above `let dispatched = …`: the first
  ("Phase 5 (§3.4): classify what this frame must recalculate. The first frame
  over an empty tree is always full…") describes the *plan*, sixteen lines below;
  the second describes dispatch. Nothing flags it — `rustdoc` never sees a `//`
  comment, so this is the same defect with no tool that could ever have caught
  it. The classification paragraph is now above the classification, extended to
  name the four sources it chooses between; the dispatch paragraph stayed put.
  This is the twelfth of the family and the first that was not a `///`.
- **`let _ = dispatched;`, a residue with nothing left to silence.** It sat after
  the dispatch-plan merge to launder a binding used only inside a
  `debug_assert!`. `debug_assert!` expands to `if cfg!(debug_assertions) { … }` —
  the tokens are compiled in every profile, so the binding was always used and the
  discard was always dead. Removed in the new home rather than carried, in the
  `dump_virtual_geometry` tradition.
- **A blank line between a comment and the code it describes**, before
  `let phases_run`. Harmless to any tool, but it is the same shape that produces
  the `///` defect. Closed.
- **Two doc links that resolved only through an import that had to go.** The
  `diagnostics` **field**'s doc links `[`NullDiagnosticSink`]` and
  `[`ResourceReporting`]`, and both resolved because `ui_runtime.rs` imported
  those types for `new`'s body and the resource resolutions. With the bodies gone
  the imports are unused, and removing them silently breaks a link on an item that
  did not move. Both are now link-reference definitions in the field's own `.md`,
  the fix the facade pass established. Five more of the same in the moved docs
  (`FrameScheduler` and three of its methods, `SceneCoordinator::compose_with_images_scrolled`,
  `ScrollTargets::fold_point`, `ScrollOffsets`, `FramePlan`). The identical
  56-warning `cargo doc` output before and after is what says all seven landed.
- **The test module needed two imports of its own.** `Stylesheet` and
  `FrameResult` were named by `run_frame`'s signature and by nothing else in the
  parent file; the tests spell both constantly through `use super::*`. They are
  imported in `mod tests` with a comment saying which child module names them
  now — the pattern that file already uses for six other types. **No test was
  changed**: the suite is 6,229 before and after.
- **One pre-existing broken link, deliberately left.** `request_background`'s doc
  links `[`FrameRequest`]`, which has never resolved (it is in the baseline
  `cargo doc` output). It travelled with its method rather than being fixed in
  passing, exactly as `set_root_view`'s private-item link did last pass.

`showcase_png` renders `showcase_hidpi.png` at `bb153594…`, a digest this
document has recorded before. Per the caveat at the top of this file that is
evidence for "this step changed nothing", not for a chain — and here the real
proof is that `run_frame` **is** the scene path, so the 137 integration test
files that assert paint output against the spec all run through the moved code.

## What the next reader inherits

The bar, and 1,346 lines of `impl` to get there. The remaining bulk is three
subjects, and each is a stage:

| what is left | lines | shape |
| --- | --- | --- |
| pointer + gesture + drag | ~460 | `handle_pointer` (254) is eleven labelled steps; the pointer pass already split its *recognizers* out, so what is left is the step sequence |
| text editing | ~500 | 20 methods, and the campaign has already refused extracting them as an **entity** — moving them into a child module is a different question and is not blocked by that refusal |
| keys, a11y publication, scroll, DPI | ~390 | `handle_key` (79), `handle_ime` (77), and the rest are small |

Moving all three leaves the struct, its 47 field docs, the module declarations,
`is_activation_key` and the `Debug` impl — **under 200 lines**, which is the
first time this file's arithmetic has closed under the bar without an argument
about what cannot move.

Two things learned here that the next stage should not rediscover:

1. **`pub(super)` is the whole visibility story**, and it costs nothing. Reach
   for it the moment a moved private method has a caller in another file; do not
   reach for `pub(crate)`.
2. **Check `cargo doc` against a saved baseline, not against zero.** The crate
   emits 56 warnings for reasons that predate this campaign, so the only usable
   signal is a diff. Capture it *before* editing.

---

# The input pass: the pointer, the key and the gesture leave the facade — and the register empties

**Status:** the input surface has left `ui_runtime.rs`, and with it the last file
in the workspace over the bar. The file went from **1,542 to 769 lines of
non-test code** (3,186 → 2,417 total); its `impl UiRuntime` from **1,346 lines
over 61 methods to 583 over 35**. Five child modules were created and three
existing ones grew, **768 non-test lines** in the new files, the largest 308 —
all of them clear of the bar by a factor of three or more.

The consequence is the one this arc was for: `xtask`'s `file-size` rule reports
`ui_runtime.rs` as *under* the 1,000-line cap and demands its entry be deleted,
so **`KNOWN` in `tools/xtask/src/rule/file_size.rs` is now empty**. The register
that existed to hold one debt holds none, and the rule that used to describe the
god object now only guards against a new one.

Suite **6,229 passed / 0 failed** (unchanged — no test was moved, weakened or
deleted), `clippy` **363 → 362** (one warning fixed, none added), `fmt` clean,
`xtask check` and `xtask consumer` both pass, and `cargo doc -p obscur-runtime`
emits **the same 56 warnings as before** — the check that matters when bodies
move, because a doc link resolves in the scope its item now sits in.

## What moved, and into which module

| module | methods | non-test lines | the subject |
| --- | --- | --- | --- |
| `pointer_dispatch.rs` *(new)* | 5 | 308 | the pointer event's journey: where it landed, what it hovered, what it pressed, what it released, and what that changed |
| `gestures.rs` *(new)* | 6 | 141 | §6.16's pan, hold and fling: arming them on a contact, feeding them moves and ticks, acting on what they conclude |
| `scrolling.rs` *(new)* | 6 | 122 | CSS Overflow 3 per-element containers: the host's read and write, the wheel that routes to one, and §6.15's reveal |
| `keyboard.rs` *(new)* | 1 + 1 free fn | 99 | the three things a key press can be — an edit, a shortcut, an activation — and the order they are tried in |
| `composition.rs` *(new)* | 2 | 98 | §5.12 IME: the four host events applied to the session, and the live preedit read back out |
| `command_routing.rs` *(+5)* | 24 | 195 | the dispatch half joined the declarative half: `activate` and the four `dispatch_*` entries |
| `pointer.rs` *(+4)* | 9 | 86 | capture, and the §6.9 drag surface — `capture_pointer`, `pointer_capture`, `begin_drag`, `finish_drag` joined the five delegates already there |
| `interaction_state.rs` *(+1)* | 4 | 36 | the three fields the state pass refused — and now their one writer, `recompute_pseudo_states` |

Three of the groupings are worth their reasoning:

- **The dispatch half of the command surface went to `command_routing.rs`, not
  to the input modules that call it.** `dispatch_command_event` is the one place
  §2.3's listener walk and §2.6's routing chain meet, and everything above it —
  `dispatch_command` from a host call, `dispatch_command_from` from a
  programmatic one, `activate` from a click or an Enter/Space press,
  `dispatch_text_input`/`dispatch_change` from a field's own edits — differs only
  in the envelope it builds and the binding it resolves. Splitting them across
  the pointer, the keyboard and the editing tier is how a second dispatch path
  gets invented beside the first.
- **`recompute_pseudo_states` went to the fields, not to its caller.** It reads
  `hover`, `focused` and the outstanding presses together and projects them into
  the `:hover`/`:focus`/`:active` a selector matches. Its only caller today is
  the pointer dispatch, which is a fact about events; what the method is *about*
  is the three fields the state pass refused to extract, and they are in
  `interaction_state.rs`.
- **Scrolling is one file for four entry points** because every one of them ends
  in the same two calls — derive the container set from the current boxes, then
  move it through `ScrollOffsets` — and the derivation-per-call is load-bearing:
  a node that stopped overflowing since the last frame is simply not found, and
  the write is a no-op.

## The one body that decomposed, and the four that did not

The previous pass's caution was that `handle_pointer` — 254 lines of eleven
labelled steps — "should decompose into named steps before it moves". It did,
into four, and the reasoning is the same one `run_frame` was read with:

- **`press_landed`, `drag_in_flight`, `release_landed` and `cancel_landed` came
  out.** Each is a self-contained answer to "what does this *kind* of event
  do?" — a press seats focus, the editing session, the press anchor and the
  gestures; an in-flight data drag consumes the event instead of sharing it; a
  release turns into a click, an activation and the end of everything the contact
  was carrying; a cancel discards the press *without* a click. Each has a narrow
  interface (the event, the point, the target, one out-parameter) over internals
  that touch half the runtime.
- **The ordering did not, and that is `handle_pointer`'s remaining content.**
  Hover is built after the pan has had its say, because the move that begins a
  pan is also the move that captures the pointer; focus moves only on a press; a
  drag in flight returns before the click path can also activate what it was
  dropped on; the pseudo-state recompute runs once at the end rather than after
  each of the five things that can change it. Every question the sequence asks is
  already an entity in its own file — `PointerLandingResolver`, `HoverUpdate`,
  `GestureArming`, `PointerSessions`, `PointerRelease`, `CommandDispatch`,
  `StructuralStates` — so what is left after them is the order, and the order is
  the thing.
- **`handle_key`, `handle_ime`, `arm_gestures` and `advance_flings` moved
  whole.** `handle_key` is 79 lines of *arbitration* — edit first, then chord,
  then activation, and the frame request skipped for a consumed edit — and each
  of the three things it arbitrates between is already elsewhere; carving it
  would leave three one-line calls and take the reasons with it. `handle_ime` is
  a four-arm match onto `TextEditSession` plus the follow-up the runtime owes,
  and the follow-up is a value precisely because the session borrow must end
  first. `arm_gestures` and `advance_flings` are each one loop over what a
  recognizer already decided.

## What this pass found

- **A `clippy` warning that had been riding in a doc file.** `handle_key.md`
  numbered a step `3b.`, which is not a markdown list marker, so
  `doc_lazy_continuation` fired on it. It is now an indented continuation
  paragraph of step 3 — same words, same rendering intent, one warning fewer
  (363 → 362). Fixed in the new home rather than carried over, per the campaign's
  rule for moved code.
- **One import the test module needed.** `NamedKey` was named by
  `is_activation_key` and by nothing else in the parent's code; the caret-scroll
  and textarea tests press `Home` and `Enter` through it. It is imported in
  `mod tests` with a comment saying which child module names it now — the same
  pattern that file already uses for eight other types. **No test was changed**:
  6,229 before and after.
- **No misplaced doc comment.** Every moved attribute sat on the item it
  described; the campaign's count of that defect stays at twelve.
- **Every `.md` moved with its item, and nothing was left behind.** A check that
  every `include_str!` in each of the 28 files resolves and that every `.md` in
  each `.doc/` directory is referenced by exactly one item comes back clean.

## What the next reader inherits

**Not the bar** — that is met, and `xtask` now enforces it with an empty ledger.
What is left in `ui_runtime.rs` is 769 lines: the struct with its 47 field docs
(~100), the 27 module declarations, the imports, `Debug`, and an `impl` of 583
lines over 35 methods. Two thirds of that `impl` is one subject:

| what is left | methods | lines | shape |
| --- | --- | --- | --- |
| text editing | 14 | ~335 | the session's lifecycle — seed, reseed, commit, the pointer's caret, the key's edit, and the two caret-scroll re-derivations. The campaign refused extracting it as an **entity**; a child module is a different question |
| the ports and motion | 8 | ~120 | the shaper/itemizer/resolver/hyphenator swaps, the locale, and the two `animate` entries |
| the environment, a11y publication, surface, DPI, inspection, virtual geometry | 13 | ~135 | small, and each already delegates to an entity |

Moving the first of those three would take the file under 450. Whether it
*should* move is now a question about the file's shape rather than about a rule:
the number that forced the campaign is no longer being violated.

---

# The sweep: the `impl` empties, and `ui_runtime.rs` becomes the struct

**Status:** `impl UiRuntime` no longer exists in `ui_runtime.rs`. All **35
remaining methods** moved into child modules — four new, six grown — and the
file went from **769 to 176 lines of non-test code** (2,417 → 1,834 total). What
is left is the `#![doc]`, the imports the 47 fields need, **31 module
declarations**, the struct with its field docs, and `Debug`. The largest child
module in the directory is still `pointer_dispatch.rs` at 308; the four new ones
are 148, 66, 55 and 46.

Suite **6,229 passed / 0 failed** (unchanged — no test was moved, weakened or
deleted), `clippy` **362 → 362**, `fmt` clean, `xtask check` and `xtask
consumer` both pass, and `cargo doc -p obscur-runtime` emits **the same 56
warnings as before**.

## What moved, and into which module

| module | methods | non-test lines | the subject |
| --- | --- | --- | --- |
| `text_edit_commands.rs` *(new)* | 4 | 148 | applying an `EditCommand` to the session, and the three things owed once one has been applied |
| `text_pointer.rs` *(new)* | 2 | 66 | the pointer's half of text selection: the press that seats a caret, the drag that extends it |
| `text_ports.rs` *(new)* | 5 + 1 private | 55 | the four outgoing text ports a host installs, and the language they shape in |
| `caret_visibility.rs` *(new)* | 2 | 46 | keeping the caret inside its field, on both axes (CSS Overflow 3 §3.1) |
| `text_editing.rs` *(+7)* | 9 | 146 | the **life** of the focused session: where it comes from, what re-seats it, when it commits, what reads it |
| `surface.rs` *(+4)* | 13 | 112 | the environment write joined the geometry it is about, with the surface/scale pair and the DPI invalidation they share |
| `motion.rs` *(+3)* | 6 | 96 | starting and advancing a timeline joined the rules and the two counts |
| `diagnostics.rs` *(+2)* | 9 | 69 | the §7.6 inspect *writes* joined the §7.6 inspect reads |
| `accessibility.rs` *(+4)* | 7 | 67 | the three pushes joined the two projections and the bridge query |
| `virtual_lists.rs` *(+2)* | 6 | 64 | the two §4.9 entries that need the boxes and the scroll offsets joined the four that do not |

## Why the editing surface became four files and not one

It was 14 methods and ~335 lines — comfortably one file under the bar, which is
exactly why the bar was the wrong question to ask of it. Asked §3.16's question
instead — how many subjects does this hold? — it answers four, and each of the
four has a boundary that can be stated in one sentence:

- **`text_editing.rs` decides *which* buffer is live.** Seed on focus, reseed
  from a controlled frame, commit on blur, tear down on leaving. Nothing in it
  changes what is *in* a buffer.
- **`text_edit_commands.rs` changes what is in one.** `apply_edit` is the single
  choke point every mutation flows through — typed insert, backspace, paste, IME
  commit — and that is precisely what makes WHATWG's readonly rule enforceable
  in one place instead of at four entry points. `after_edit` is the tail all of
  them share.
- **`text_pointer.rs` is the pointer's adapter onto that path.** It turns a
  press or a drag into a `SetSelection` and lets the ordinary edit path do the
  rest; routing it that way is why a click asks for a frame and fires no
  `on-input` without either behaviour being written twice.
- **`caret_visibility.rs` is the one subject that spans editing and scrolling.**
  Its two methods are a pair rather than one because the axes are seated in
  different owners: a single-line field's horizontal offset belongs to the
  **session**, a `<textarea>`'s vertical offset belongs to the **scroll map**,
  because a multiline field is an ordinary overflow container a wheel can move
  too. Seating the vertical one on the session would give a textarea two
  `scroll_top`s that disagree.

The keyboard, IME and pointer paths were already three files (`keyboard.rs`,
`composition.rs`, `text_pointer.rs`) before this pass finished; all three now end
in the same `after_edit`, in a fourth. That convergence is the argument: what
they share is a *tail*, not a theme.

## Why `text_ports.rs` is a file and `set_locale` is in it

The four port setters share an invariant none of them states alone: **the
retained measure and arrange results were produced by the port being replaced.**
A different shaper wraps to different sizes, a different size is a different
position, and no computed-style diff can see either — so the swap has to say so
itself. It was said four times, in four identical five-line comments; it is now
said once, in `invalidate_shaped_caches`, which is a private local function with
no independent semantics of its own (§3.1 admits exactly that).

`set_locale` is the fifth member and the interesting one, because it feeds the
same orchestrator and yet deliberately does **not** clear the caches. Its own
documentation already carried the reason — the language is a member of
`LayoutInputs`, so the next frame's classifier runs a full layout, which clears
them anyway, and clearing here would be a second mechanism for one guarantee.
That asymmetry is the first thing a reader of the file will ask about, which is
the argument for keeping the two next to each other rather than filing the
locale with the styling reads.

## What stayed in `ui_runtime.rs`, and why

Two things, and neither is a method:

- **The struct and its 47 field docs.** This is the entity.
- **`impl fmt::Debug`.** It is a trait impl over the struct's own fields — the
  type's summary of itself, not a call the facade makes — and it names four
  fields and nothing else. Moving it would put the struct's `Debug` somewhere
  the struct is not.

Nothing was blocked. No borrow-checker obstruction appeared: every body moved
verbatim, because a child module sees the private fields with exactly the
visibility the parent has. Twelve of the moved private methods are called from a
*sibling* child module (or, in `edit_is_compose_only`'s case, from the parent's
own `mod tests`) and became `pub(super)` — visible to `ui_runtime` and its
descendants, and to nothing outside it. Four stayed fully private because their
only callers moved with them: `after_edit`, `publish_accessibility_now`,
`invalidate_dpi_variants`, and the new `invalidate_shaped_caches`. **Not one
field's visibility changed, and nothing became `pub(crate)`.**

## What this pass found

- **No misplaced doc comment.** Every one of the 35 moved items carried a
  `#[doc = include_str!]` naming its own `.md`; the campaign's count of that
  defect stays at twelve. Worth recording *why* the streak is now safe: since
  the doc bodies moved out of `///` blocks and into per-item files, the failure
  mode that produced all twelve — a missing blank line silently re-attaching a
  comment to the next item — is no longer expressible.
- **A doc-link regression, caught and closed.** Two links resolved in
  `ui_runtime.rs`'s scope and would not have resolved in their new one:
  `[EVENT_INPUT]` in `focus_accepts_text.md` and `[LayoutInputs]` in
  `set_locale.md`. Both are now written with an explicit path, same visible text,
  and `cargo doc` is back to the baseline 56. This is the check that matters when
  bodies move and the one no other gate performs.
- **One sentence that stopped being true.** `set_locale.md` said its sibling
  setters drop the measure cache "by hand"; after the helper they drop it
  explicitly but not by hand. Corrected rather than left to rot.
- **Four `_module.md` files that had described their own absence.**
  `motion.doc` said "starting and advancing a timeline is not here",
  `accessibility.doc` said "the push itself is not", `diagnostics.doc` said
  "nothing in this file changes what the runtime renders", and
  `text_editing.doc` said "everything that changes a session stays with the
  frame". Each was true when written and false the moment the method arrived;
  all four are rewritten to argue for what the file now holds. A module doc that
  is a list of exclusions ages badly, and this pass is the evidence.
- **`set_inspect_mode` is the one member of `diagnostics.rs` that is not free.**
  Enabling the overlay is the only thing in that file that changes what a frame
  paints, which is why it is also the only thing there that requests one. The
  module doc now says so instead of claiming the file is inert.

## What the next reader inherits

`ui_runtime.rs` at **176 non-test lines**, the smallest it has been since the
runtime had a body, and 31 child modules of which the largest is 308. Every
file in the directory is under a third of the bar.

The remaining shape questions are about the *children*, not the parent, and none
of them is urgent:

- `frame_pipeline.rs` (316) and `pointer_dispatch.rs` (308) are the two largest,
  and both are sequencing over entities that already exist elsewhere — the same
  argument that kept `handle_pointer`'s ordering together. The order is the
  content.
- Five files hold fewer than 50 lines each (`interaction_state.rs`,
  `view_source.rs`, `queued_input.rs`, `caret_visibility.rs`,
  `frame_classification.rs`). Small is not the same as wrong: each answers one
  question, and merging them by size would re-create exactly the "next 300
  lines" grouping this campaign exists to undo.
- The 1,658-line `mod tests` in `ui_runtime.rs` is now **90% of the file**. §3.1
  keeps a unit test with its entity and these drive `UiRuntime`'s public API, so
  they belong to the struct; but many of them are integration-shaped and have
  siblings in `crates/facade/obscur-kit/tests/`. Whether they should move is a
  question about the *test* layout, which no pass of this campaign has asked.

# The close: the campaign verified against the tree, not against its own reports

**Status:** every claim the four `ui_runtime` passes made was re-measured from
the working tree, trusting none of the reports. All of them hold. Two carried
lints were fixed; nothing else moved.

## What the workspace measures

Measured over **1,911 `.rs` files**, counting only lines above `#[cfg(test)] mod
tests`:

| | |
| --- | --- |
| files over the 1,000-line bar | **0** |
| the largest file in the workspace | `obscur-runtime/tests/touch_gestures.rs`, **971** |
| the largest non-test file | `obscur-kit/examples/wireframe/scene.rs`, **932** |
| `ui_runtime.rs` | **176** non-test (1,834 total, 1,658 of them tests) |
| its 31 child modules | **3,024** non-test in total, largest `frame_pipeline.rs` at **316** |

This is the first time in the campaign that the answer to "which files are over
the bar" is *none*. The register in `tools/xtask/src/rule/file_size.rs` is
`&[]`, and `KNOWN.md` already says why — no edit was owed there, because the
input pass deleted the entry the same pass that earned it, which is what the
rule requires.

## The four things worth checking, and how they were checked

**The public API is byte-identical in name.** `impl UiRuntime` blocks were
brace-matched out of `ui_runtime.rs` and all 31 children, test modules stripped,
and every `pub fn` collected — then the same done to `git show HEAD:…
/ui_runtime.rs`, the pre-campaign file. Both sets are **139 names**, and `diff`
of the sorted lists is empty. Not one public method was lost, renamed or gained
by being moved into a child module. The 34 `pub(super)`/`pub(crate)` methods are
correctly excluded from that count: a restricted `fn` is not public API, and
`pub(super)` from a child of `ui_runtime` is exactly the scope a private method
in `ui_runtime.rs` already had.

**No field was widened.** The struct body was brace-matched and all 47 fields
read: **none carries any visibility qualifier at all** — not `pub`, not
`pub(crate)`, not `pub(super)`. The mechanism the facade pass bet on held for
all four passes.

**Every doc file resolves.** All 270 `include_str!` references across
`ui_runtime.rs` and its 31 children point at a file that exists; each is
referenced **exactly once**; and the 270 `.md` files present across the 32
`.doc/` directories have **zero orphans**. The correspondence is a bijection.

**No intra-doc link broke in the move.** `cargo doc -p obscur-runtime --no-deps`
reports **zero `unresolved link` warnings anywhere under `ui_runtime`**. The
crate's two unresolved links are in `orchestration/grid_areas.doc` and
`orchestration/grid_lines.doc`, both pointing at `LayoutStyle`, and neither file
was touched by this campaign. The eight doc warnings that do name a
`ui_runtime` file are all the *other* class — `public documentation for X links
to private item Y` — which renders the text and merely declines to make it
clickable. Seven have direct ancestry in the pre-campaign file (`Self::run_frame`
appears four times at HEAD, `Self::invalidate_dpi_variants` and
`crate::orchestration::direction_for` once each).

## The one substitution worth naming

`set_locale.md`'s `[LayoutInputs]` was a shortcut link that resolved only
because `ui_runtime.rs` imported the type for its own code; the sweep rewrote it
as the explicit path `crate::runtime::LayoutInputs` and reported `cargo doc`
back at baseline. It is back at baseline, but by substitution rather than by
repair: `LayoutInputs` is `pub(crate) use` inside a private `mod runtime`, so it
is not publicly linkable by any spelling, and the fix turned one *unresolved
link* warning into one *private item* warning. Same count, different class, and
the rendered prose is identical. This is not a defect — there is no public path
to link to — but a reader diffing warning **counts** would have seen nothing at
all, which is the reason to write it down.

## The two lints that were fixed rather than counted

The passes reported "none of it from the new files". Two warnings did in fact
point into child modules, and both were carried verbatim from the pre-campaign
file rather than introduced:

- `text_editing.rs` — `collapsible_if` on `commit_editable_change`'s
  `if rebaseline { if let Some(session) = … }`, present at HEAD in the same
  shape. Collapsed to a let-chain, which edition 2024 takes.
- `frame_requests.rs` — `redundant_closure_for_method_calls` on
  `peek_frame_reason`'s `.map(|request| request.reason())`, identical at HEAD.
  The path is spelled in full rather than imported, and a comment says why: the
  reader is `#[cfg(test)]` and the only item in the file that names the request
  type, so an import would be unused in every other profile.

The campaign's own standard is that a lint is fixed in its new home rather than
carried over, so these were the last two debts of the move. **Workspace clippy
362 → 360**, and **no child module of `ui_runtime` now emits a single warning**.
The two remaining warnings that cite `ui_runtime.rs` itself are at line 1,411 —
inside `mod tests`, and older than this campaign.

## The state this closes on

`fmt` clean, `xtask check` and `xtask consumer` both pass, **6,229 passed / 0
failed** across the workspace (the number every pass of this arc has reported —
no test was moved, weakened, rewritten or deleted at any point), clippy **360**,
and no file in the workspace over the bar.

What the next reader inherits is unchanged from the sweep's account, and the two
open questions it names are still the right ones: whether `frame_pipeline.rs`
(316) and `pointer_dispatch.rs` (308) are sequencing or entities, and whether
the 1,658-line `mod tests` — now 90% of `ui_runtime.rs` — belongs to the struct
or to `crates/facade/obscur-kit/tests/`. Both are questions about shape, not
about the bar; the bar is met everywhere.
