# obscur.kit_v2 — Layer-Surface Consumer Audit

- **Date:** 2026-08-07, second pass 2026-08-08, third pass 2026-08-08, fourth
  pass 2026-08-11, **closure pass 2026-08-28**. **Audited at:** `b791b97` plus the uncommitted fix for 1-A.
  **Toolchain:** 1.96.1.
- **What the fourth pass was looking at, and on what.** The first three passes
  ran on the development machine, where the keyboard is fast enough that nothing
  about the loop's shape shows. This one ran on the target: an aarch64 tablet —
  Snapdragon 865, Adreno 650 on Turnip, Arch Linux ARM, Hyprland — reached over
  ssh, with the application tracing every action it takes to a file. The
  question was "why does it feel slow", which is not a question a faster
  machine can be asked. It produced 1-D, and 3-D is what made 1-D take longer to
  find than it should have.
- **What the third pass was looking at.** The application had reached the point
  of transcribing a finished visual design — a mockup with hover, press, latched
  and locked states on every control. That is a harder use of the styling and
  input paths than the first two passes made, and it is where 1-C and 2-C came
  from. Both had been mistaken for careless transcription for two working
  sessions before either was isolated.
- **Vantage point — and how it differs from the audit next door.**
  `external-consumer-audit.md` was written from `../test-app`: a separate crate
  depending only on the facade, opening an ordinary **window**. This one is
  written from `../obscur.virtual-keyboard`: a separate crate depending only on
  the facade, opening a **desktop surface** — `App::layer(LayerSettings)`. The
  application is an on-screen keyboard, which is the use `LayerSettings`' own
  documentation names first.

  That is a different vantage, not a repeat of the same one. The two runners
  diverge after `AppParts::boot`, and everything below is in the half of that
  divergence a window never reaches.
- **The layer path had no consumer inside this repository — closed 2026-08-08.**
  The same structural reason `external-consumer-audit.md` gives for finding 1-A
  applied here one level down: `checks/facade-consumer` closed the "no consumer
  outside the facade" gap, and it opens a window. Nothing in this repository
  built a layer surface *and* used the authoring surface on it, so finding 1-A
  below survived the refactor that introduced it.

  There is now a second `xtask consumer` profile, `checks/layer-consumer`: a
  crate outside the workspace whose one dependency is the facade **without
  `window-winit`**, composing `App::layer` end to end and mounting a user
  component through two nested `<For>` loops. It carries the mechanical half of
  this audit's own diagnosis — the build log is scanned for warnings under
  `crates/facade/obscur-kit/src/app/`, so the `fields ... are never read` that
  1-A identified as the defect speaking now **fails the check** instead of
  scrolling past in a build nobody runs. Verified by re-introducing an unread
  field and watching it fail. Cold cost: 141 s and 2.6 GB for both profiles.
- **Method.** Build the application; where behaviour was in question, isolate it
  before attributing it. Finding 1-A was diagnosed by first proving the
  application's own half correct — the view tree, the props on it, and the
  component id the tag hashes to are asserted in
  `../obscur.virtual-keyboard/crates/keyboard-ui/tests/view_shape.rs` — and only
  then reading the runner. Each finding is evidenced by `file:line` or by a
  compiler message.
- **Not done here.** The workspace test suite was not run; no claim rests on
  test counts.
- **Tiering** follows `conformance-audit.md`: **Tier-1** = actively wrong or
  blocking. **Tier-2** = spec-required, real gap. **Tier-3** = documentation
  defect.

---

## Status

| # | Status |
|---|---|
| 1-A | **closed** 2026-08-07 — the layer loop drives `AppControl` on the same two beats the windowed one does |
| 2-A | **closed** 2026-08-07 — `App::displays()` answers before `run`, so a name can be resolved to a `DisplayId` |
| 3-A | **closed** 2026-08-28 — the diagnostic states the rule the check actually enforces. A method call on a field is refused because its shape is indistinguishable from service access, not because it *is* service access, and `expression_validator.rs` now says the former. `09-authoring-formats.md` and `12-reality.md` no longer describe it as a known defect |
| 3-B | **closed** 2026-08-07 — documented in the `okui!` `<For>` paragraph |
| 1-B | **closed** 2026-08-28 — a refused property is named in words. The derive carries the field's name beside the symbol, so the report reads ``property `step` (#2147483681) could not be read as u32`` where it used to read `property #2147483681 …` — a number that cannot be turned back into a name |
| 2-B | **closed** 2026-08-28 — the row axis read the *offered* space where the column axis read the stated percentage basis, so a `height: 100%` grid measured with an unbounded block axis fell through to an infinite content height and every `1fr` row resolved to infinity. Both axes now resolve against `constraints.percent_*` |
| 1-C | **cause disproved** 2026-08-28 — the runtime has asked for the frame itself, on every hover, focus and press change, since `e0792634` (2026-08-01); the layer runner needs no `request_frame` of its own and the finding's stated cause cannot be the one that was observed |
| 2-C | **closed** 2026-08-28, and it was three defects stacked. (i) Every shorthand expander split a value into longhands *before* lowering, and each eagerly-parsed longhand judged it against a grammar no function can satisfy — so one `var()` sank the whole declaration. A value that is one reference is now *carried* as `CustomRef` and judged at computed-value time, which is the only place that can judge it. (ii) A custom property kept only the **first component** of its value — `--font-body: Inter, sans-serif` compiled to a lone interner-local keyword, `--shadow: 0 2px 4px #000` to `Number(0.0)` — so even a reference that arrived resolved to nothing; a custom declaration now keeps the text it was written with, per CSS Variables 1 §2, and is parsed when it is substituted. (iii) A substitution that does not fit the property it lands on now computes to `unset` — the inherited value for an inherited property, the initial for one that is not — rather than reading as *absence*, which is a behaviour CSS does not offer |
| 2-D | **closed** 2026-08-12 — layout and paint summed the same advances in different orders and disagreed by 1.3e-4 px; every fit decision in the packer now goes through one tolerance |
| 2-E | **closed** 2026-08-08 — the present blit wrote into an sRGB view, encoding what Vello had already encoded; colours measure at the authored values now |
| 2-F | **half closed** 2026-08-28 — an application can now name a font. `FontResolver::resolve`'s family loop could only match faces already loaded, with no channel to ask anyone for a family it had not heard of; there is one now, answered by a host-registered face or by fontconfig, and an author sheet's `font-family: "X"` reaches it. **Still open**, and explicitly the finding's second half: per-character fallback picks by coverage alone, so an emoji-presentation code point still lands on whatever monochrome outline happens to carry it |
| 2-G | **closed** 2026-08-28 — the real half. The property model had one `Overflow` id and no axis longhands, so `overflow-y: auto` resolved to no property at all and was dropped before any scroll or clip code could see it. The longhands are now the storage and the shorthand expands into them. (The withdrawn half stands withdrawn: `overflow: auto` did scroll on block, grid and flex alike.) One limit is documented rather than fixed: a longhand asking for a scroll on one axis alone is accepted, not reported, because a compiled declaration is either `Standard` or `Unsupported` and there is no "accepted with a caveat" |
| 2-H | **closed** 2026-08-12 — a reconfigure asks for a frame only when it changes the extent; a move is a commit, and `LayerSettings::needs_redraw_from` is where that is decided |
| 3-C | **closed** 2026-08-08 — a debug build installs a stderr sink by default, and `App::diagnostics` replaces it |
| 2-I | **closed** 2026-08-28, two of three, and the third does not reproduce. The cell-height special case handed a single `auto` row's item back its own measured height instead of the row's, against css-grid-2's "by default, grid items stretch to fill their grid area"; and grid read **no `margin` at all** — `git show HEAD:…/grid_layout.rs | grep margin` returns nothing — so a margin on a component's root had nothing to be dropped *by*. Both are fixed, and grid stopped clamping an item's border box back inside its cell. The third sub-defect (a stated size without a grid collapsing to the glyph) was re-run on today's tree and lays out correctly; it is not pinned, because pinning a defect that is not there proves nothing |
| 2-J | **narrowed, not closed** 2026-08-28 — no fix, because the evidence names no line and a guess here would be a change nobody could defend. What measurement *did* settle: it is not the viewport mapping (`surface_geometry.rs`) and not the rasteriser — a shape tangent to a fractionally scaled surface keeps its extreme rows all the way through the CPU path, at every scale tried. The swapchain present blit and the compositor's `wp_viewport` scaling are both still unexamined and neither is reachable from any test in this workspace; settling it needs a real Wayland session and `grim`, which is how the pixel was found in the first place. The null result is published with a positive control — a check that deliberately drops a column and asserts the comparison notices — because without one, three passing assertions would prove nothing |
| 1-D | **closed** 2026-08-12 — the blocking acquire is off the input path on **both** runners, and the win is now measured on the target: an event's wait went from median 36 ms / p90 333 ms / worst 544 ms to **0.9 / 2.0 / 61.6** |
| 3-D | **closed** 2026-08-09 — the runtime's subtotal is called `build` and says present is not in it; the runner prints `waited`/`build`/`present` with the worst present kept |
| 1-E | **closed** 2026-08-12 — the swapchain was configured before `sync_scale` declared the buffer's scale and never again until a reconfigure; it is now configured after, and a frame whose image disagrees with the window's extent is dropped rather than blitted. The same defect also **panicked the process** six times that day (`Surface::configure` while an image was live); confirmed gone on the target |
| 2-L | **partly closed** 2026-08-28, and one of its three claims did not survive. **Fixed:** the desktop-surface loop advanced animation by the interval since the last *frame* while moving the frame mark only on a turn that acquired an image, so every turn in between was handed the same elapsed time again — eight turns spanning 40 ms advanced motion by 180 ms. Reading the mark and moving it are now one step (`TurnClock`). The windowed runner never had this: it moves its mark with `replace` and returns early when no image arrives, so the two stay paired. **Did not reproduce:** "more than half the frames built are never shown" — the loop does not build a frame it then discards; what it does is ask for the image at the *top* of a turn while the input that makes a frame pending arrives in the *middle*, which costs a turn's latency, not a frame. **Still open, and it is the headline:** every press and every release restyles the whole tree — 320 of 320 nodes, 5.0 ms against 1.68 ms for a hover-only frame — because `:active` is projected onto the pressed node's entire ancestor chain, which always reaches the root. Narrowing that needs a selector-dependency map and an inherited-property diff; it is a scoped piece of engineering, not a follow-up. **Caveat on the fix:** its tests exercise `TurnClock`, a type the change itself introduces, so they cannot fail without it — the defect lived inline in a loop that needs a Wayland compositor to run. It is verified by reading, not by ablation, and that is stated rather than glossed |

### The closure pass — 2026-08-28

Every finding still marked open on 2026-08-13 was worked in one sitting, one
agent per finding, each required to reproduce the defect before fixing it and to
prove its test fails without the fix. Three results are worth reading before the
sections below, because none of them is a fix:

- **1-C's stated cause was wrong**, and had been since before the finding was
  written. See its section.
- **2-I's third sub-defect does not reproduce** on today's tree, and 2-L's
  "more than half the frames built are never shown" does not either. Both were
  true observations attached to explanations that have since stopped being true;
  neither was pinned with a test, because a test for a defect that is not there
  asserts nothing.
- **2-J got no fix at all.** The evidence rules out this workspace but does not
  name a line below it, and a change that cannot be defended is worse than an
  open finding.

The pass also found what an audit written from outside cannot: **three defects
that only existed because of the fixes**. A `var()` that survives compilation
reads as *absence* rather than `unset` unless the computed-value step judges it;
a custom property that keeps its text needs a parser at substitution time, which
`KitBootstrap::install_system_theme` did not install, so the kit's own
`--obscur-sys-*` palette would have resolved to nothing for a host that never
called `CssPipeline::install`; and a new definiteness branch in flow assigned a
height without clamping it by `min-height`/`max-height`. All three were caught by
review rather than by the suite, and all three are closed. The consumer fixture
`checks/facade-consumer` now contains a `var()` for the same reason it exists at
all: the suite could not see that class of defect.

**And the pass kept pulling.** Closing 2-C's `var()` half exposed two defects
that had nothing to do with the layer path and everything to do with the
machinery underneath it:

- **The same invalid keyword, written directly, still clipped the box.**
  `overflow-x: sideways` in a hand-written sheet minted a sheet-local symbol,
  and `OverflowAxes` reads anything that is not `visible`/`clip` as clipping —
  measured, one `PushClip` against zero. The `var()` fix had bolted a *second*
  copy of the rule onto the substitution path instead of moving it, so only one
  spelling of the typo was refused, and the commoner one was not. The rule now
  lives in `lower_for_property` alone; the proof it is single-sourced is that
  reverting the literal path's line now fails the `var()` path's test too.
- **A sheet-local symbol could equal a reserved one.** `InternedSymbol`
  partitions its space by the top bit, and `CssSymbolInterner` numbered from
  **zero** — inside the reserved range. Measured, not argued: the 9219th
  interned word came out byte-identical to `SIZING_FIT_CONTENT`, which
  `length_resolution.rs` reads as `fit-content`, so a long enough stylesheet
  would have sized a box to its content because of a typo. Sheet-local ids now
  mint into the same high namespace `named()` uses.

Following that thread found one more: `layout_keyword::name_of` was an
incomplete inverse — 87 constants went out through the `*_from_name` tables and
only 71 could come back, so the sweep that asks it "is this symbol reserved?"
was answering wrongly for 23 of them, including all three the collision landed
on. Completed, and pinned by a round-trip test that asks the forward tables for
the list rather than a human remembering it.

---

### What the fixes were, and one correction to this audit

**1-A.** `layer_runner::run` now takes `booted.css` and `booted.control` and
applies the queue on both beats `runner.rs` uses: guarded before the frame is
built, and **unconditionally after it commits** — the second is the one that
resolves positional bindings, because a binding waiting for a tree is not a
*pending request* and nothing would ask on its behalf. The `LayerControl`
parameter is now `layer_control` and the `AppControl` is `app_control`, so the
two no longer sit three lines apart under one name. The dead-field warning this
audit correctly identified as the defect speaking is gone: `title` and `size`
left `BootedApp` (the windowed runner reads them from the parts before `boot`
consumes them), and a `shell-layer`-only build now compiles `obscur-kit` with
**zero** warnings.

**One claim in 1-A does not survive checking.** *"No user component can be
registered on a desktop surface"* is too strong. `AppParts::boot` calls
`control.apply` once (`app_parts.rs`), so anything queued **before** `run` — which
is every `with_runtime`, including a `register_component`, since `run` blocks —
was already drained on the layer path. What was genuinely dead is everything
queued *after* boot: post-boot `set_stylesheets`, and `on_command_at` /
`add_event_listener`, whose targets resolve against a committed tree that does
not exist at boot. Those are now live. If the keyboard's keys were missing for a
different reason, this fix will not have found it — worth re-running before
attributing.

**Re-run, and that caveat was right.** With 1-A fixed the keyboard still drew no
keys. The cause is 1-B below, and the two are unrelated. The control experiment
that separated them took one run and should have been the first thing tried:
`App` is one entry for both surfaces and `layer(settings)` is the whole of the
difference, so running the same application **in a window** answers "is this the
layer path?" immediately. It drew nothing there either. Recorded because the
shape of the mistake is the useful part — a defect found on the way to a symptom
is not thereby the cause of it.

**2-A.** New `LayerProbe::displays()` in the adapter opens a short-lived
connection, binds `wlr-layer-shell` (so an unsupported desktop answers the same
way it would from `run`), does the one roundtrip that names the outputs, and
drops the connection. Ids come from the same `display_id(index)` mapping
`LayerWindow::open` resolves against, so they agree. The facade exposes it as
`App::displays()`, and both it and `LayerWindow::displays` now return
`LayerDisplay { id, name }` rather than a tuple. `examples/panel.rs` takes a
connector name (`panel eDP-1`) and resolves it, instead of asking a person for a
position in the compositor's discovery order.

What this audit **checked and found working**, so the next reader need not:
`okui!` compiles against the facade alone (1-A of the other audit stays closed);
`<For>` nests, and its body may be a component tag with expression props;
`App::layer` opens, configures and presents; `LayerFocusPolicy::Never` is the
default, which is the correct default for a keyboard and removes the one setting
such an application cannot afford to forget.

---

### 2-H — a move is a commit, not a repaint (2026-08-12)

Taken as the report proposed. The decision is a domain method,
`LayerSettings::needs_redraw_from(previous)`, rather than a condition in the
runner: what changes the pixels is a property of the settings, and putting it in
the loop would have left the same judgement to be re-made by the next backend.

A frame is asked for when the **extent** changes — `width`, `height`, or the
`anchor`, since anchoring both edges of an axis is what stretches a surface — or
when the **display** does, because a different output may bring a different
scale. Band, margin, reservation and focus policy get a commit and nothing else.

Asked *before* `apply`, since the question is what changed and `apply` overwrites
the surface's record of what it was living under.

**A real resize still repaints**, and not because of this: the compositor sends a
configure whenever the geometry it chose has changed, and the loop already turns
that into a frame. What this removes is asking for one *in advance* on the
strength of a guess — a guess that was wrong for every margin in a drag.

Pinned in the domain (six cases, including the control that the method does not
simply answer `false`) and from outside the facade in
`checks/layer-consumer`, whose own `docked()` / `floating()` differ by exactly a
margin and a reservation. The `panel` example still renders unchanged after the
change: 497 colours, dominant `(34,49,67)`.

---

### 2-D — closed, and it was one comparison (2026-08-12)

**The layout boxes were right all along.** For the audit's own sentence at 14px
on an 800px surface, `inspect_snapshot` reports the text box as `294 × 14` — one
line, correct width. Only the display list disagreed, with two `DrawText` runs.
So this was never "wraps against a width that is neither its own nor its
container's": layout wrapped correctly and **paint re-wrapped it wrongly**.

Instrumenting `PackedLine::pack` gave the whole thing in one line:

```
[pack] max_width=293.9999
[pack]   word w=92.40001 trailing_ws=-0 current=201.60002 -> break=true
```

`201.60002 + 92.40001 = 294.00003 > 293.9999`. Layout measures the run **once**
and sizes the box from that total; paint re-wraps against that very box,
rebuilding the total **word by word**. Same additions, different order, and
`f32` addition is not associative — the disagreement is 1.3 × 10⁻⁴ px.

That also explains the two things the report found inexplicable. It only
happened with a rule that changes the text's own metrics, because only then does
the box come from a fresh measurement whose rounding differs. And it moved with
the font size rather than the container because the error scales with the
magnitude being summed, which is why 14px and 28px both failed while 10px and
16px did not — a float coincidence, not a formula.

**Fix.** One helper, `overflows(width, max_width)`, used at all three fit sites
in the packer, comparing against a relative `FIT_EPSILON` of `1e-5` — about
eighty times the observed error and under a tenth of a pixel at ten thousand
pixels wide. Relative, not absolute, because the error grows with the sum.

Verified: 14px and 28px now paint one run of 294px and 588px; a 200px surface
still breaks the same sentence in two, so genuine overflow is untouched. Four
regression tests in `crates/facade/obscur-kit/tests/text_fits_its_own_box.rs`,
including the control (an unstyled span, which never misbehaved) and the
overflow case a careless tolerance would have broken.

**Two corrections to the report.** `width: 700px` alone does *not* trigger it —
only a rule that changes the text metrics does; and the workaround table's
`white-space: nowrap` was treating a symptom, so the place it had to be set no
longer matters.

---

### 1-E — the swapchain was built before the scale was declared (2026-08-12)

**The audit's own account was half of it.** Two threads do now touch the surface,
but that is not what changed: the runner this one replaced had the identical
setup order — `init_gpu`, then `sync_scale`, then a loop that resized only on a
configure event. Reading for a race found nothing that the old code did not also
have.

The difference is one line I added and thought was tidy. The old runner left the
initial configure event in the queue; its **first loop turn** therefore took the
`resized` branch and called `surface.resize(…)`, which always calls
`configure(…)` even at an unchanged extent. So the old build rebuilt its
swapchain once, *after* `sync_scale` had committed `set_buffer_scale` /
`viewport.set_destination`. The new runner drains that event before the loop
("whatever arrived during the wait was the configure itself") and so configures
the swapchain exactly once, in `init_gpu`, **before** the surface says anything
about its scale. That is also why any later reconfigure repairs it for the rest
of the session: a reconfigure is the only other path to
`VelloSurface::resize`.

**Two changes.** `layer_runner` now configures the swapchain immediately after
`sync_scale` and before the first image is ever requested, via a
`resize_to_surface` the configure branch shares. And both runners now compare
`AcquiredImage`'s extent against the window's before the blit: a disagreement
drops the image and asks for a resize, leaving the frame pending. That second
half is what the audit asked for — the mismatch is caught at the blit instead of
on the screen — and it holds whatever the cause turns out to be.

**Honesty about the cause.** The ordering above is the leading account, not a
proven one. It was not reproduced here: this machine has no 2× layer surface to
open, `checks/layer-consumer` is headless, and `AcquiredImage` holds a
`wgpu::SurfaceTexture` and cannot be built without a GPU — so **1-E has no
automated test**, and the guard is the thing that makes the class of failure
non-visual rather than a proof that the cause is understood. The consumer's
tablet is what can confirm it.

---

### 1-D — the acquire moved off the input path, on both runners (2026-08-12)

The first of the two shapes named above, and the one the spec's wording points
at. `SurfaceAcquirer` (`obscur-render-vello`) owns the surface and makes the
blocking call on a thread of its own; `present_to_texture` is the half of
presenting that takes an image somebody else acquired.

**It is a pull, not a stream.** Nothing is fetched until the loop asks, so an
idle application does not sit holding a swapchain image the compositor could be
using. `AcquiredImage` carries the extent and format it was acquired at, because
the surface is no longer reachable from the loop and a blit against the wrong
extent is a stretched frame with nothing to catch it.

**A frame with no image is skipped, not waited for** — §3.5, "stale intermediate
frames may be skipped". The image is taken *before* the frame is built, so a
turn that finds none discards no work and leaves the pending frame pending;
`run_due_frame_reactive` is simply not called.

**Waking differs by runner, and the difference was a real bug.** The layer loop
gets a second `LayerWaker` through the new `LayerWindow::add_turn_waker`, whose
source pushes nothing: it means "your `dispatch` may return", not "reconsider
the interface", and a `Woken` there would have cost a frame per acquire. The
windowed loop uses an `EventLoopProxy`. The first attempt used
`Window::request_redraw` from the acquire thread — **the window never appeared
at all**, because that marks the window dirty without rousing a loop parked in
`wait`, which is exactly where a skipped frame leaves it. A proxy user event
does both.

**Verified on a live compositor, both paths.** `panel` maps and paints (498
distinct colours in the strip, dominant `(34,49,67)`, 0.5% light pixels — the
text). `hello_app` maps and paints, and its three colours measure
`(45,55,77)` / `(210,85,83)` / `(91,180,93)` — identical to the 2-E measurement
taken before this change, so presenting and the sRGB fix both survived.

**What is not verified is the win.** The 544 ms burst was measured on a tablet
under a finger; here both examples idle and never reach the 60 turns
`[layer-loop]` reports on. What is demonstrated is that the blocking call is no
longer on the thread that reads the socket. Confirming the latency is the
consumer's trace to take, and it is the same "take it twice" the fix direction
asked for.

**Still open:** `present_to_surface` remains, acquiring inline, for hosts with no
thread to spare — its doc now says plainly that a host reading input on that
thread must not use it. And the runners still name `VelloRenderer` directly
rather than the `Renderer` port, which is a separate finding waiting to be
written: the port has no consumer outside the adapters.

---

### 1-D — the win, measured under a finger (2026-08-12)

The trace the section above asked the consumer for, taken twice on the same
tablet with the same application: once on the build that reported 1-D, once on
the build against the fix. Nothing else changed on the device between them.

| an event's wait between the compositor's stamp and the handler | before | after |
|---|---|---|
| median | 36 ms | **0.9 ms** |
| p90 | 333 ms | **2.0 ms** |
| worst | 544 ms | **61.6 ms** |
| bursts of ≥4 events delivered inside 10 ms | 4 in 66 events | **none in 114** |

The measure is the gap between the compositor's own stamp on an event and the
handler running, quoted against the smallest gap seen in the session, so it is
queueing above best case and nothing else. The application's own work is
unchanged and remains a rounding error: a whole keypress handler costs 0.052 ms
(worst 0.112), the view builds in 0.30 ms.

The runner's own counter agrees from the other side — the present that used to
hold the thread now costs what a submission costs:

```
[layer-loop] 60 turns | waited 8.50ms | build 0.94ms | present 0.57ms (worst 10.67ms) | turn 10.01ms
```

**What the fix did not do, and could not.** The finger is heard immediately; it
is not *answered* immediately. First repaint after a keypress is 1.2 ms at the
median but **46.7 ms at p90 and 234 ms at worst**, and while typing the surface
redraws about **twelve times a second** on a 120 Hz panel. That is 2-L below,
and it was invisible while 1-D was in front of it.

---

### 1-D is a spec violation, and it is not the layer path's alone

Raised after the fix above, and it changes what 1-D is. The spec forbids this
shape in three separate places:

- **Core principle 5** (`specs/…/README.md:39`): *"Minimal blocking. I/O,
  loading, decoding and **platform calls are asynchronous**. Synchronous work is
  short, deterministic and **frame-bounded**."* `get_current_texture` is a
  platform call, it is synchronous, and 544 ms is not frame-bounded.
- **§3.4 Frame pipeline**: *submit/present* is step 12 of the enumerated
  pipeline, and the section closes *"The synchronous frame phase must stay short
  and deterministic."* It is neither.
- **§9 Non-goals**: additions *"should be rejected or redesigned if they …
  **require hidden synchronous blocking**"*. Hidden is exact: nothing in the API
  says that presenting stops the loop reading input.

**What the spec does not forbid is vsync.** §3.5 lists *"frame callbacks/vsync
are used where available"* among the scheduler's rules, so pacing to the display
is endorsed. The two rules are compatible only if the wait happens **off the
path that reads input** — which is the whole content of the defect, and the
reason `Immediate`/`Mailbox` is a workaround rather than the answer.

**Correction to this finding's own framing.** It says the escape hatch "exists on
the path that does not need it and is missing from the path that does". The
windowed path needs it too: `runner.rs`'s `window_event` handles input *and*
`RedrawRequested → render_frame → present_to_surface` (`:335`) on one thread, so
while the acquire blocks, winit dispatches nothing either. Same mechanism, same
violation. The layer path did not introduce it — an on-screen keyboard is simply
the first consumer that touches the surface often enough to feel it.

So this is not a layer-runner bug to be patched locally. It is how the kit
presents, on both surfaces, and §9's word for that class is *redesigned*.

**Two shapes that would satisfy the spec**, neither yet built:

1. **Acquire off the loop thread.** `wgpu::Surface`, `Device` and `Queue` are
   `Send`; the acquire-and-present moves to a worker, the loop hands it a
   finished display list and never waits. Costs a frame of latency in the
   pipeline and a queue discipline for skipping stale frames (§3.5 already
   permits "stale intermediate frames may be skipped").
2. **Dispatch while an image is outstanding.** The loop keeps the acquire but
   pumps the connection between asking and drawing. Smaller, and it only shrinks
   the window rather than closing it — the block is still on the thread.

The first is the one the spec's wording points at. Recorded rather than chosen:
it is an architecture decision, not a patch.

---

### What was done on 2026-08-09

**3-D — closed.** Two halves. The runtime's line no longer calls its subtotal a
frame: it reads `N builds | … | build 28.28ms (present is NOT in this)`, and the
comment at the format string says where the rest of the time went. And the layer
runner prints its own line on the same cadence, gated on the same
`OBSCUR_FRAME_STATS`:

```
[layer-loop] 60 turns | waited 1.20ms | build 2.10ms | present 402.55ms (worst 544.01ms) | turn 405.85ms
```

Three phases kept apart rather than summed, because the shape of 1-D is that one
of them is enormous and the others are fractions — a total hides exactly that,
which is also why `perf::FrameStats` (two numbers) was not reused. The **worst**
present is carried beside the average for the same reason: a burst averaged over
sixty turns reads as healthy.

`waited` counts only a *bounded* wait taken while a frame was pending. An idle
`dispatch(None)` is the loop asleep with nothing to do, and counting it would
swamp the number and mean nothing.

**1-D — step 1 only, and it is not the fix.** `layer_runner::init_gpu` now has
the `OBSCUR_NO_VSYNC` branch the windowed runner has had all along, so the
diagnosis can be confirmed or refuted on a given machine and the escape hatch is
no longer missing from the one path whose purpose is to be touched.

**What that does not do is fix the burst.** With vsync on — the default, and
what ships — `present` still blocks in `get_current_texture` and the socket is
still unread while it does. Step 3 remains the honest fix and it is a real
change: the acquire has to leave this thread, or the loop has to dispatch the
connection while an image is outstanding. Recorded here rather than quietly
counted as done.

**Not verified on hardware.** The measurement that would close this needs the
target tablet under a finger; what is verified here is that the branch compiles,
that the three phases are timed at the right points, and that the counter keeps
a worst case. The audit's step 1 says "take the trace twice" — that is the
consumer's to run.

---

### What was fixed on 2026-08-08, and how it was checked

**2-E — the double sRGB encode.** Cause located exactly where the measurement
pointed. Vello rasterises into an `Rgba8Unorm` texture whose bytes are *already*
sRGB-encoded; `present_to_surface` blitted them into a view created with
`TextureViewDescriptor::default()`, i.e. the surface's own
`Bgra8UnormSrgb` — and a write to an sRGB view applies the transfer function in
hardware. Fixed in `VelloSurface`: `new` lists the suffix-stripped format in
`view_formats`, `blit_format()` names it, and the present blits through a view in
that format. Put there rather than at the call sites because a host that forgot
would get the defect with no error attached.

**This is also why no test caught it.** `render_to_image` — the headless Vello
path every golden PNG uses — reads back from the `Rgba8Unorm` frame directly and
never touches the blit. Headless output was correct all along; only what reached
a screen was wrong, and nothing in the suite looks at a screen.

Verified by the audit's own method, `grim` on the composited output of
`examples/hello_app`:

| authored | double-encoded would be | measured |
|---|---|---|
| `#2b3348` (43,51,72) | (114,124,145) | **(45,55,77)** |
| `#d9534f` (217,83,79) | (238,155,151) | **(210,85,83)** |
| `#5cb85c` (92,184,92) | (162,221,162) | **(91,180,93)** |

Within 6–13 units of the authored colour across three channels — the residue is
the capture's 1.6× fractional-scale resampling — against 166–206 units from the
prediction this finding was built on.

**3-C — diagnostics into a sink nobody installed.** Took the third of the
audit's directions, which it called the cheapest and best. New
`obscur_kit::bootstrap::StderrDiagnostics` (floor: warnings and worse), installed
by `AppParts::boot` **in a debug build only** — first, before the stylesheets
compile, since that is the loudest producer. A release build keeps the silent
default. `App::diagnostics(sink)` replaces either.

Deliberately not in `UiRuntime::new`: a library must not print, and the runtime
is embedded by hosts that have their own reporting. The standalone entry point is
the one place where "nobody installed a sink" means "nobody will".

---

## Tier-1 — actively wrong or blocking

### 1-A. ✅ CLOSED — `AppControl` was never applied on the layer path

Closed; what the fix was, and which of this finding's claims did not survive
checking, are above. It was **not** the cause of the symptom that opened this
audit — see 1-B.

The finding as first written:

`App::control()` is callable on a layer application, accepts queued operations,
and none of them ever run. The queue is the only route to
`UiRuntime::register_component`, so **no user component can be registered on a
desktop surface** — and because nothing checks a component name
(`09-authoring-formats.md`, "The typo that stays silent"), the tag mounts as an
anonymous box with no diagnostic.

Observed as: a keyboard that draws its shell, no keys, and reports nothing.

**Cause.** The two runners take different fields out of the same `BootedApp`.

```
app_parts.rs:38-46      BootedApp { runtime, css, boot, title, app_id, size, control, scroll }

runner.rs:115-116       css: booted.css,
                        control: booted.control,   // <- both taken
runner.rs:285-286       if self.control.has_pending() {
                            self.control.apply(&mut self.runtime, &mut self.css);
runner.rs:303           self.control.apply(&mut self.runtime, &mut self.css);

layer_runner.rs:49-51   let mut runtime = booted.runtime;
                        let boot = booted.boot;
                        let mut scroll = booted.scroll;
```

`booted.control` and `booted.css` are dropped. `grep -c AppControl
crates/facade/obscur-kit/src/app/layer_runner.rs` → `0`.

Nothing is missing from the plumbing: `AppParts` already carries
`control: AppControl` (`app_parts.rs:30`), `layer_runner::run` already receives
`parts: AppParts` whole (`layer_runner.rs:25`), and `parts.boot(...)` already
produces it. The value arrives and is not used.

**The build says so already.** In a `shell-layer`-only feature set — no
`window-winit`, which is what a keyboard builds with — the compiler emits:

```
warning: fields `css`, `title`, `app_id`, `size`, and `control` are never read
  --> crates/facade/obscur-kit/src/app/app_parts.rs:40:16
```

That warning *is* this defect. It is invisible in a `full` build, where
`runner.rs` reads every one of those fields.

**Blast radius beyond components.** Everything `AppControl` carries goes the same
way. `AppControl::set_stylesheets` — closed as 2-D in the audit next door — is
inert here too, so a layer application cannot replace a sheet after boot.

**Fix direction.** Take the two fields and apply the queue, as the windowed
runner does. In `layer_runner::run`, after the `parts.boot` at `:48`:

```rust
let mut css = booted.css;
let control = booted.control;
```

and inside the loop, alongside the existing `control.take_pending()` at `:143`
(which is the *`LayerControl`*, a different object with a similar name — worth
distinguishing at the call site, since the two now sit three lines apart):

```rust
if app_control.has_pending() {
    app_control.apply(&mut runtime, &mut css);
}
```

**Worth checking on the same branch.** `screen()`, `screen_data()` and
`on_command_at()` arrived in the same refactor that folded `LayerApp` into `App`.
`screen` is resolved before the branch (`app.rs:281-290`) and so is shared; the
other two were not separately exercised by this audit and the windowed runner is
the only place they are known to work.

---


### 1-B. ✅ CLOSED — a props mismatch fails a component's render, silently

**Closed 2026-08-28, on the third of the three directions below — the one that
does not change the authored language.** The failure was never entirely mute:
`ComponentRenderPass::report_failure` builds a real `Diagnostic`, and since 3-C
a debug build installs a stderr sink by default, so it *is* printed. What it
printed was `property #2147483681 could not be read as u32`.

That number is an `InternedSymbol`: an FNV-1a hash with the high bit set
(`interned_symbol.rs`), and there is no reverse table anywhere in the toolkit —
by design, because the whole point of interning names is to stop carrying them.
So the report named the property in the one alphabet its reader cannot spell.

The name survives in exactly one place: the literal `#[derive(Contract)]` read
off the field. It now travels with the symbol. `PropsError`'s two variants each
carry a `label: &'static str`, `decode_required`/`decode_optional`/`decode_value`
take it, and the derive passes the field's contract name. The message reads

```
property `step` (#2147483681) could not be read as u32
required property `key_id` (#…) is missing
```

— the number kept beside the name, because every other report about the same
property prints the number and a reader correlating two of them needs the two to
agree on something. Decoding by bare symbol is still legal and falls back to the
number alone.

Pinned by `error_boundaries.rs`'s
`the_report_names_the_property_in_words_not_only_by_number`, which fails with the
label removed and prints exactly the old message when it does; and by
`okui_name_binding_drift.rs`, which now pins the label alongside the symbol, so
the derive cannot drift from the name an author writes.

**Deliberately NOT done: kebab-case is still not snake_case.** `key-id` does not
reach `key_id`; it fails, and now says which name it wanted. Accepting both
spellings would make one field answer to two names, and this codebase has an
explicit rule against aliasing. The compile-time direction — teaching §2.13's
unknown-attribute check to see a *known* component's fields — remains the better
fix and is still open; it is a larger change to the okui compiler, and it is not
needed to stop the defect costing an afternoon.


A component whose props do not convert **mounts, never renders, and reports
nothing**. Its host node stays empty, so the tag draws as an anonymous box —
the same outcome as an unregistered tag, from a different cause, with the same
absence of any message.

Observed as: a keyboard with ninety keys in its tree, ninety instances mounted,
and nothing on screen.

**The mismatch that produced it.** A hyphenated attribute does not reach a
snake_case field:

```rust
#[derive(Contract)]
#[contract(name = "Key")]
pub struct KeyProps { key_id: String, /* … */ }

okui! { <Key key-id={id} … /> }   // mounts, never renders
okui! { <Key key_id={id} … /> }   // renders
```

Both parse — `6a24c73` made `okui!` read `ident ('-' ident)*` so §7.5 ids reach
the primary front end — and only the second converts. The attribute name has to
equal the field name exactly.

That is a defensible rule. What is not is that breaking it costs nothing at
compile time and produces no message at run time, in the one place where §2.13
already checks the *attributes of a known component* — this **is** a known
component, and the attribute is unknown to it.

**Why nothing is said.** `MountedComponent::render` converts props before
calling the component; a failure returns `GuardedRender::Failed`,
`ComponentRenderPass::report_failure` constructs a `Diagnostic`, and it goes to
the null sink no shipped host installs — the case `12-reality.md` already
records as "reachable, not reached". Here it is the difference between a
two-minute fix and an afternoon.

**Why a mounted count does not catch it.** `mounted_component_count()` counts
instances, and mounting is the step *before* the conversion: the instance is
there, the render is not. An application's own test asserting `> 0` passes while
nothing draws — which is what happened, and why the reproduction in
`../obscur.virtual-keyboard/crates/keyboard-ui/tests/component_mounts.rs` now
asserts on the display list instead.

**Fix directions.** Any one closes it. Report the conversion failure where §2.13
already reports an unknown attribute on a known component — at compile time, on
both front ends, which is where a name mismatch belongs. Or map kebab-case to
snake_case, so the markup's natural spelling reaches the field. Or, at minimum,
make the run-time diagnostic reach a default sink, so an application that never
installed one still learns its component refused its props.

---

### 1-C. ⚠️ CAUSE DISPROVED — pointer input never asks for a frame on the layer path

**Re-checked 2026-08-28: the cause named below is not the cause.** The paragraph
"`handle_pointer` marks no frame pending of its own" was wrong when it was
written. `UiRuntime::handle_pointer` ends with

```rust
if hover_changed || focus_changed || press_changed {
    self.recompute_pseudo_states();
    self.request_frame(FrameReason::Input);
}
```

(`runtime/ui_runtime/pointer_dispatch.rs`), and has since `e0792634`, 2026-08-01
— a week *before* this finding was written. It is pinned by
`obscur-runtime/tests/pointer_dispatch.rs`'s "hover change requested a frame"
and its negative twin `unchanged_hover_does_not_request_a_frame`.
`handle_scroll` requests one too, at the end of `handle_wheel`. The windowed
runner's extra `request_redraw` is winit bookkeeping, not the frame request; the
layer runner needs no counterpart, and adding one would have been a change that
did nothing.

The adapter half is fine as well: `layer_state.rs` maps SCTK's `Enter`, `Leave`
and `Motion` to `PointerEventKind::Enter`/`Leave`/`Move`, so hover crossings do
reach the runtime on a desktop surface.

So the observation — no `:hover` on a layer surface, taps working — stands
unexplained, and it should not be closed on this reasoning. What this
re-check can say is where it is *not*: not in the runner's frame request, not in
the runtime's response to a crossing, not in the adapter's event vocabulary.
The one structural oddity left in the loop is that the swapchain image is asked
for at the **top** of a turn while the input that makes a frame pending arrives
in the **middle** of it — so a frame first requested by a pointer event waits a
turn before its image is even requested. That is latency, not a lost repaint,
and it belongs to 2-L.

**Re-run it on the target before attributing anything to the layer path.** The
audit's own lesson applies to its own finding: a defect found on the way to a
symptom is not thereby the cause of it.


`:hover` and `:active` do not exist on a desktop surface. The styles compute,
the pseudo-states are projected, and nothing repaints — so every hover and press
state an application writes is dead on the one surface an on-screen keyboard
uses.

**Cause.** The two runners treat pointer input differently, and only one of them
is right.

```
runner.rs:506-507       self.runtime.handle_pointer(event.with_timestamp(self.now_micros()));
                        self.request_redraw();
runner.rs:523-524       self.runtime.handle_pointer(left);
                        self.request_redraw();
runner.rs:534-535       self.runtime.handle_pointer(event);
                        self.request_redraw();

layer_runner.rs:99-101  LayerInput::Pointer(pointer) => {
                            runtime.handle_pointer(pointer);
                        }
```

The windowed runner asks for a redraw after **every** pointer event, and says
why at `CursorLeft`: without it a `:hover` outlives the pointer. The layer
runner calls the same door and asks for nothing. `handle_pointer` marks no frame
pending of its own — the loop below it only builds when `has_pending_frame()` —
so a hover crossing restyles a tree that is never painted again until something
unrelated requests a frame.

The scroll branch three lines down has the same shape and, one suspects, the
same problem: `handle_scroll` is called and no frame is requested.

**Isolated before being attributed.** The same binary, same view, same
stylesheets, with and without `App::layer(settings)` — the application keeps
that switch precisely for this
(`../obscur.virtual-keyboard/crates/keyboard-app/src/main.rs`, `as_window`).
Windowed: hovering a toolbar button draws its accent border and tint. Layer:
nothing, at any position, while taps on the same buttons work. Taps work because
they change application state, which wakes the surface through `LayerWaker` and
requests a frame for a different reason.

**Fix direction.** Request a frame after pointer and scroll input, as the
windowed runner does:

```rust
LayerInput::Pointer(pointer) => {
    runtime.handle_pointer(pointer);
    runtime.request_frame(FrameReason::UiCommit);
}
```

Better, if the runtime is to stay the judge of what needs redrawing:
`handle_pointer` already returns a `PointerDispatch` — if that carried "the
restyle changed something", both runners could ask for a frame on the same
evidence instead of one of them remembering to.

---

### 1-D. ✅ CLOSED — the layer loop read no input while it waited for a swapchain image

On the target machine the keyboard periodically stops answering the finger for
a third to half a second, then answers everything at once. It is not slow — it
is deaf, in bursts.

Measured with the application's own trace, which writes a line for the start and
the end of every action it takes. Four taps on the same key, and what the loop
did with them:

```
45825.284 > key.up    key=char:г waited=544.011      ← stamped 544 ms ago
45826.542 > key.down  key=char:г waited=453.268
45827.874 > key.up    key=char:г waited=387.601
45828.919 > key.down  key=char:г waited=313.646
45830.450 > key.up    key=char:г waited=257.176
45831.652 > key.down  key=char:г waited=182.379
45832.974 > key.up    key=char:г waited=125.700
45834.001 > key.down  key=char:г waited=51.728
45834.494 > frame                                    ← the first frame in all of it
```

Eight events spanning **544 ms of the user's time, delivered inside 9 ms**,
oldest first, and one frame after them. `waited` is the gap between the
compositor's own stamp on the event and the handler running, quoted against the
smallest gap ever seen in the session, so it is queueing above best case and
nothing else. Over 66 events: median 36 ms, p90 333 ms, worst 544 ms. Between
bursts the same measure reads 1.4 ms.

**Cause.** The loop is one thread and does four things in sequence
(`layer_runner.rs:70-167`):

```
layer_runner.rs:74-77    let timeout = runtime.has_pending_frame()
                             .then(|| Duration::from_millis(ANIMATION_STEP_MS));
                         window.dispatch(timeout)?;       // ← the only place input is read
layer_runner.rs:97-114   for input in window.poll_input() { … }
layer_runner.rs:140-158  if runtime.has_pending_frame() { … present_to_surface(…) }
```

and the last of those blocks:

```
vello_renderer.rs:299-301    let surface_texture = surface.current_texture()…
vello_surface.rs:58-59       match self.surface.get_current_texture() {
```

`get_current_texture` waits for a free image in the swapchain, and the layer
path runs the swapchain on **`Fifo`** — vsync — because `init_gpu`
(`layer_runner.rs:202-241`) takes `surface.get_default_config(&adapter, w, h)`
and passes it through untouched. So the wait is "until the compositor has
composited this surface and released the buffer", and for as long as it lasts
the socket is not read, `poll_input` has nothing to return, and the touches
queue in the kernel.

**The windowed runner already knows this.** `runner.rs:721-736`:

```rust
// `get_default_config` picks `Fifo` — vsync — which is the right default …
// But it also makes `present` block until the next refresh, so a profiling run
// cannot tell real GPU cost from waiting …
if std::env::var_os("OBSCUR_NO_VSYNC").is_some() {
    let supported = surface.get_capabilities(&adapter).present_modes;
    for mode in [wgpu::PresentMode::Immediate, wgpu::PresentMode::Mailbox] { … }
}
```

The layer runner has no such branch, so `OBSCUR_NO_VSYNC` does nothing on a
desktop surface — the one surface where the blocking matters most, because it is
the one whose whole purpose is to be touched. The escape hatch exists on the
path that does not need it and is missing from the path that does.

**Isolated before being attributed.** The application's half was measured first
and is not in it: on the same trace, a whole keypress handler costs **0.091 ms
at its worst** (median 0.047), the facade mutex is taken in 0.001 ms, the view
builds in 0.29 ms, and every side effect — committing to the field, synthesising
keys, Hyprland IPC — was moved to a queue drained by a thread of its own, whose
depth never exceeded one batch. Nothing the consumer does is on the path of a
touch. What remains is a window of hundreds of milliseconds in which the loop
neither drew nor read, bracketed by a frame at each end.

**Fix direction**, cheapest first, and only the first is a patch:

1. **Prove it.** Give `layer_runner::init_gpu` the same `OBSCUR_NO_VSYNC` branch
   the windowed runner has — six lines — and take the trace twice. If the
   bursts go, it is this.
2. **`Mailbox` where it is supported.** A swap, not a fix: it stops the waiting
   by drawing frames nobody sees, which on a battery-powered tablet is a worse
   bargain than it looks.
3. **Read input while waiting.** The honest one. `present` leaves the input
   path: either the acquire moves off this thread, or the loop dispatches the
   connection while the image is outstanding. The keyboard reached the same
   shape one level up for the same reason — the finger must never wait behind
   an acknowledgement — and it is a queue and a thread, not a redesign.

Neither 2-H nor 1-C is this one. 2-H is a repaint asked for when nothing
changed; this is the cost of the repaint's *last* step, paid by the input path,
and it would remain after every unnecessary frame in 2-H had been removed.

---

### 1-E. ✅ CLOSED — since the acquirer took the surface, the first frames came out at double extent

The keyboard opens drawn **twice as large**, clipped at the right edge and the
bottom: seven key columns fill a width that holds thirteen. It appeared with the
2026-08-12 kit and was not there the day before.

**What it is not.** Layout is right and the compositor agrees with it: the
surface reports `0 474 1472 446` at `scale: 2.00` on a `2944x1840@120` panel,
which is the geometry the application asked for. **Hit-testing lands on the
right keys** — a finger on what looks like the wrong place types what the layout
says is there. So the tree, the styles and the arrangement are all correct, and
what is wrong is only what reaches the screen.

**What repairs it.** Any change of presentation mode. Switching dock ⇄ float
puts the drawing right for the rest of the session — and a mode change is a
reconfigure, which is the one thing that re-runs `sync_scale`
(`layer_runner.rs:130`) and, with it, `set_buffer_scale` /
`viewport.set_destination` (`layer_window.rs:212-227`).

That pair — correct layout, correct hit-testing, wrong only until a reconfigure
— places this at the buffer's declaration and not in the pipeline above it. The
comment guarding it is still in the runner and still describes the symptom
exactly:

```
layer_runner.rs:87-91
    // Declare how the buffer relates to logical pixels before the first frame,
    // or the compositor reads the physical buffer as a logical size and the
    // panel comes out at double extent with everything drawn twice as large.
    window.sync_scale();
    runtime.set_surface(window.physical_size(), window.scale());
```

Both calls are still there and still before the loop. What changed underneath
them is who owns the surface and when it is first configured:
`SurfaceAcquirer::spawn` (`layer_runner.rs:85`) now takes it, configures it on
its own thread, and hands back images acquired at the extent *it* holds — while
`sync_scale` commits the `wl_surface` from the loop's thread. Two threads now
touch the same surface's state, and the first presented buffer is the one that
gets it wrong.

**Not blocking once it is up.** The consumer keeps typing: taps land, text
commits, and one mode switch puts the picture right.

**But it did not always come up — the same defect panicked the process.** Added
after the closure above, from the consumer's journal on the day: **six of these**
before the fix, none since.

```
wgpu error: Validation Error
Caused by:
  In Surface::configure
    `SurfaceOutput` must be dropped before a new `Surface` is made
…
thread 'main' panicked at wgpu-core-29.0.4/src/storage.rs:135:46:
Surface[Id(0,1)] does not exist
obscur-keyboard.service: Main process exited, code=exited, status=101/n/a
```

Which fits the closure exactly and is the same fault seen from the other end:
the swapchain was being configured at a moment it may not be, so the configure
failed validation and the surface it should have produced was not there when the
next turn asked for it. The wrong scale was the survivable half of it.

**It was survivable only because the consumer runs under a supervisor.** It came
up on the third and fourth attempt. Where it stopped being survivable was the
restart limit — five restarts in a minute, spent in full, left a tablet **with no
on-screen keyboard at all**, which is also no way to type the command that brings
it back. That unit is the consumer's own and has been widened; it is recorded
here because an application on this runner needing a restart policy to survive
its own toolkit is not a sentence that should be true.

**Confirmed fixed on the target.** The build carrying the 13:55 change has been
up since with `NRestarts=0`, no validation errors in the journal, and the
keyboard draws thirteen key columns at the stylesheet's own size.

**Fix direction.** Establish the buffer's relation to logical pixels *before*
the acquirer is given the surface, or give the acquirer the same duty on
configure that the loop has: `AcquiredImage` already carries the extent it was
acquired at, so the mismatch is detectable at the point of the blit rather than
by looking at the screen. A test that catches it needs one frame's pixels and
the logical size to compare against — `checks/layer-consumer` already presents;
what it does not do is assert that a drawn key is the size the stylesheet says.

---

## Tier-2 — a real gap

### 2-A. A display cannot be chosen before `run`

`LayerSettings::with_display(DisplayId)` is the only way to pin a surface to a
monitor, and `DisplayId` is the right currency for a platform-neutral API — the
docs say so, and they are right that a connector name identifies a monitor on
Wayland and nothing on the next platform.

But an id is only obtainable from a backend that is running:
`WindowBackend::displays` (`window_backend.rs:111`) and
`LayerWindow::displays` (`layer_window.rs:190`, which does return
`Vec<(DisplayId, String)>` with the name in it) both need an open connection,
and `App::layer(settings)` takes its settings **before** `run` opens one. There
is no `App::displays()`, and `AppControl` — which might have carried a
post-boot reconfiguration — is finding 1-A.

**Why it matters here rather than for a panel.** Left to the compositor, "the
output" on a multi-head desktop is whichever one had focus when the surface was
created. Observed: the keyboard opened on the external monitor on one run and
the laptop panel on the next, with nothing changed between them. For a panel
that is untidy. For a keyboard it means the keys are on a different screen from
the field being typed into.

What an application can pass today is the position in the compositor's discovery
order — `DisplayId::from_raw(index + 1)`, per `layer_state.rs:137-140`. That is
an implementation detail of one backend, and it is a poor thing to ask a person
for.

**Fix directions.** Any one would do: expose the display list before `run`
(`App::displays()`, or a free function on the layer backend); or let
`LayerSettings` name a display and have the backend resolve it, keeping
`DisplayId` for callers that already hold one; or close 1-A, so a running
application can be told which display to move to.

---


### 2-B. `grid-template-rows` does not divide a `height: 100%` grid container

A keyboard is naturally a grid of grids: rows down the surface, keys across a
row, with each key's width a track in the row. The second half works exactly as
it should — `grid-template-columns: 1.5fr 1fr 1fr …` inline on a row gives every
key its share, and a grid cell stretches to its cell, which a flex item's child
does not.

The first half does not. On a container that is `height: 100%`:

```css
.keyboard { height: 100%; display: grid; gap: 6px; }
```

- `grid-auto-rows: 1fr` — every row takes its content height, and the first one
  fills the surface;
- `grid-template-rows: repeat(4, 1fr)` — the same;
- `grid-template-rows: 1fr 1fr 1fr 1fr` — the same.

Replacing the container with `display: flex; flex-direction: column` and
`.row { flex: 1 1 0% }` divides the same height into four correctly, which is
what the keyboard ships. So this is not the rows being indefinite in some way
flex also sees.

It is plausibly the same root as the row `12-reality.md` already carries —
"every algorithm but flow clamps its own border box", which names grid among the
four — and if so this is the case that makes it visible rather than a new
defect. It is recorded separately because the symptom is different in kind: not
an overflow of a few pixels but a track list that does not divide at all.

**Fix direction.** Whatever closes the clamping row for grid, checked against a
`height: 100%` container with `1fr` rows in it — the numbers that row calls
unasserted are exactly these.

---

### 2-I. Three ways to inset a component in a grid cell, and none of them work

Measured on 2026-08-08, laying out one round button on a small layer surface.
The button had to sit **two pixels inside** the surface — a circle whose radius
is exactly half its box is tangent to all four sides, and a shape tangent to the
edge of what it is drawn into loses its extreme row and column downstream. Three
ways to leave that room; all three come back wrong, and each wrongly in its own
way.

The container is a grid of one cell holding one component:

```css
.shell  { width: 100%; height: 100%; display: grid;
          grid-template-columns: 1fr; align-items: stretch; }
.button { width: 100%; height: 100%; }   /* the component's own root */
```

**1. `padding` on the container shifts the cell but does not size the track.**
With `padding: 2px` on a 60×60 shell, the button is laid out at `x=2` — so the
padding reached placement — and comes back **60 wide**, overhanging the
container by two pixels on the right. Either the track is sized from the border
box while the origin comes from the content box, or the percentage below
resolves against the wrong one; the pair is inconsistent whichever it is.

**2. `margin` on the component's root is dropped on the cross axis.** With
`margin: 2px` and `width: 56px; height: 56px` on the button in a 60×60 stretched
cell: `x=2, y=2, w=56, **h=58**`. The horizontal margin and the stated width
both hold; the stated height loses to the stretch, and 58 is exactly
`60 − 2` — the cell height less the top margin. The host node the toolkit puts
between a parent and a component appears to stretch its child's height and not
its width.

**3. A stated size on the component's root, without the grid.** Replacing the
grid with a centred flex row and giving the button `width: 56px; height: 56px`
gives it back **14 wide** — the width of its glyph. This is the same case
2-B ends on and `12-reality.md` names: the host takes the component's intrinsic
size and the root cannot grow past it. It is recorded here because the workaround
for it (a grid, so the *cell* stretches the host) is exactly what (1) and (2)
then break.

What does work: state the size on a **plain element** — the view's own root,
which has no host above it — and let the component fill it.

```css
.shell  { width: 56px; height: 56px; margin: 2px; display: grid; … }
.button { width: 100%; height: 100%; }
```

`x=2, y=2, w=56, h=56`. Correct, and arrived at by elimination rather than by
anything the documentation says.

**Fix direction.** (2) and (3) are the host-node question and probably one
defect: a host should pass a stated size through and should not stretch one
axis while ignoring the other. (1) is separate and cheaper — a percentage inside
a padded grid container should resolve against the content box the track was
sized from, and today the two disagree.

---

### 2-K. An open component's internals take no rule of any kind

`#[contract(style_scope = open)]` is documented as "the component's rendered
elements are styled by the document's sheets exactly as authored elements are".
Its **root** is. Nothing beneath it is.

Measured on a suggestion candidate — a `<button class="suggestion">` containing
two `<div>`s, `.suggestion-kind` and `.suggestion-text`, rendered by a component
that declares `style_scope = open`:

| node | authored | computed |
| --- | --- | --- |
| `.suggestion` (root) | `display: flex` | `flex` ✓ |
| `.suggestion-kind` | `font-size: 10px` | 14px — inherited |
| `.suggestion-text` | `font-size: 15px` | 14px — inherited |

`computed_style` returns a style object for both children, with nothing of the
authored rule in it. The same holds for `.key-face`, `.key-sub`, `.pick-label`,
`.setting-hint` and `.drag-grip` — every internal element of every component in
this consumer.

Inline `style="…"` on the same elements is dropped too, so the workaround the
external-consumer audit names for scoped components ("inline props, which do not
exist either") is not available to an open one either.

**What it looked like.** Two lines of body text at the inherited size, hard
against the left edge of a cell 523 pixels wide, where the design has a 10px
tracked-out label centred over a 15px word. It reads as a layout bug, and every
CSS-level fix for it fails silently, because the rules being edited never
matched anything.

**The way through.** `::part(name)` crosses the boundary, and the toolkit opens
it for any internal element carrying `name` as a class — nothing has to be
exported. `::part(suggestion-kind)` applies where `.suggestion-kind` does not,
and a host qualifier works as well: `.key::part(key-face)` matches, so the host
half is checked against the classes the component put on its root.

That is a good mechanism. What is wrong is that `open` reads as "the document's
sheets reach this subtree" and does not, so the failure is silent and looks like
a layout defect rather than a scoping one. Either `open` should mean what it
says, or a component whose internals take no author rule should say so.

### 2-J. A shape tangent to the surface edge loses its extreme row — unresolved

Related to 2-I and recorded separately because it is **not** demonstrably the
toolkit's.

A 56×56 layer surface, drawn as a circle of radius 28 filling it exactly. The
CPU rasteriser is correct: rendered at 56×56, the profile matches a true circle
within half a pixel on every row, and the scene carries a proper `RoundedRect`
with 28 on all four corners and the standard circular béziers.

On screen it came out with flat left and right sides, about twenty rows tall —
measured off a `grim` capture of exactly the surface rectangle. The output is
fractionally scaled (1.6), so 56 logical is 89.6 physical and the widest point
of the circle lands where the surface's last column is not.

Leaving two pixels of room fixed it completely (`grim` again: 89×89, deviation
from a true circle at most 1.85px, all of it the antialiased fringe). So the
practical answer is "do not draw a shape tangent to the surface" — but whether
the pixel is lost in the viewport mapping, in the swapchain, or in the
compositor is not something this application can see from outside.

**Fix direction.** Worth a look at the fractional path — a logical size that is
not an integer number of physical pixels, and what happens to the last column of
the buffer. If it turns out to be the compositor's, this entry can be withdrawn
and the room documented as the rule instead.

---

### 2-C. A shorthand whose value holds `var()` is dropped whole

```css
.key       { background: var(--k-face); }        /* sets nothing */
.key       { background-color: var(--k-face); }  /* sets it */
.key:hover { background: var(--k-face-hover); }  /* an empty rule */
```

The longhand reads the variable; the shorthand does not, and the whole
declaration is discarded rather than the part that could not be read. Confirmed
for `background`, `border`, `border-<side>`, `border-color`, `border-radius` and
`box-shadow`. `font-family: var(--font-body)` fails too, and that one is a
longhand — a font stack is parsed eagerly and a variable is not a font name.

**How much this cost.** Thirty-two declarations across five stylesheets, in an
application whose CSS had been reviewed twice by eye. Among them **every hover
and press rule in the interface** — each one was `background: var(…)` and
nothing else, so each compiled to an empty block: the design's whole interactive
layer, absent, in a stylesheet that said it was there. Also every modifier's
face colour, every border colour that was not a literal, and every
`font-family`, so the interface was drawn in the fallback face.

**What it looks like from the outside.** A design transcribed carelessly. There
is no way to tell a dropped declaration from one never written, so the reading
that fits is "the person doing the transcription missed things" — and the
correct response to that reading, writing the same declarations again more
carefully, produces no change at all.

**Where it happens.** `property_compiler.rs:107-128`: `lower_for_property`
returns `None` and the declaration becomes
`CompiledDeclaration::Unsupported(UnsupportedCssFeature)` — kind
`UnsupportedFeatureKind::Function`, because the value holds a function call the
lowering for that property does not evaluate. The shorthand expanders above it
(`expand_box_shorthand`, `border_side_shorthand`) split a value into longhands
*before* lowering, so a `var()` in any position sinks the whole shorthand rather
than the one longhand it belongs to.

**Not silent — see 3-C.** The compiler reports every one of these, and
`css_pipeline.rs:57-66` forwards them as diagnostics. They are invisible only
because the default sink discards them, which is a different defect with a much
smaller fix.

**Fix directions.** Either resolve `var()` inside a shorthand before expansion —
the substitution is textual and the expanders already walk the value — or, if
that is a larger change than it is worth, lower the shorthand per-longhand so a
`var()` in the colour position does not take the width and style down with it.
Documenting the limit would be a third of a fix: an author who knows it can
write longhands, and the consumer here now does
(`../obscur.virtual-keyboard/crates/keyboard-ui/tests/stylesheet_compiles.rs`
compiles every authored declaration on its own and fails the build if one
compiles to nothing).

---

### 2-D. A styled inline element wraps its text against the wrong width

A `<span>` that carries a `font-size` or a `width` of its own wraps its text at a
width that is neither its own nor its container's, and that does not change when
the container grows.

Reproduced headless, one 35-character string at 14px, `.row` and the surface
both set to the width shown:

| container | first line | second line |
|---|---|---|
| 200px | `this field asked for ` (168px) | `no suggestions` |
| 400px | `this field asked for no ` (193px) | `suggestions` |
| 800px | `this field asked for no ` (193px) | `suggestions` |
| 1600px | `this field asked for no ` (193px) | `suggestions` |

The same string wraps identically in an 800px box and a 1600px one, and the
break moves with the **font size** rather than with the available space: at 28px
it breaks after the same 24 characters, at exactly twice the advance (386.4px).
At 10px the whole 35-character string fits on one line at 210px — which is wider
than the 193px that "did not fit" at 14px.

What decides it is whether the element has a rule of its own:

| `.label` rule | result |
|---|---|
| *(none)* | one line, 336px |
| `color: red` | one line, 336px |
| `max-width: 700px` | one line, 336px |
| `display: block` | one line, 336px |
| `white-space: normal` | one line, 336px |
| `font-size: 14px` | **wraps at 193px** |
| `width: 700px` | **wraps at 193px** |
| `font-size: 14px; width: 800px` | **wraps at 193px** |

A declared `width: 800px` does not raise the wrap point, which is the part that
makes this look like a bug rather than a policy: whatever width the line breaker
is given, it is not the box's.

**Where it was met.** A privacy note in a 1768px bar broken across two lines, and
`Русская (ЙЦУКЕН)` on a space bar 850px wide broken after `Русская`. Both
elements had nothing but a `font-size`.

**It is contagious across a flex row.** The note's neighbour — a small bordered
`PRIVACY` badge — drew as `PRIVAC`. Nothing was wrong with the badge: the row
believed it was short of room, because the note in it had been measured against
the wrong width, so it shrank the badge instead. That cost an afternoon and very
nearly a second finding against `letter-spacing`, which turned out to measure
correctly. A wrong available width does not stay in the element that got it.

**Workaround.** `white-space: nowrap`, which the consumer now sets on every
label that is a single line by design — but *where* it is set matters, and that
part is not isolated:

| declaration on | result |
|---|---|
| `.privacy-note` — the span holding the text | one line ✓ |
| `.key-face` — the span holding the space bar's label | still breaks at its only space ✗ |
| `.key` — the button that span sits in | one line ✓ |

The failing case is a span inside a component's rendered output, inside a flex
button, in a grid cell; the working cases are a plain span in the view and the
button itself. The space bar is 850px wide and the label is about 130px of it,
so nothing here is short of room. The consumer sets the declaration in both
places on that key and says why at the call site.

**Repro.** Root view `<div class="row"><span class="label">…</span></div>`, one
stylesheet, `run_due_frame_reactive`, then count `PaintCommand::DrawText` in the
display list: two commands at `y=0` and `y=14` is the wrap, and `run.text()`
gives the break point.

---

### 2-E. Authored colours are painted a stop and a half too light

Every colour in every sheet reaches the screen as `srgb_encode(colour)` — the
value is written as if it were already linear-light, and the surface encodes it
to sRGB on the way out. Nothing is being blended wrongly and no alpha is
misread; the transfer function is simply applied once too often.

Measured on this application's own palettes, with `grim` on the composited
output:

| authored | expected | measured | `srgb_encode` of authored |
|---|---|---|---|
| `--glass` dark `rgb(22,17,12)` | (22,17,12) | **(83,73,61)** | (83,73,61) |
| `--glass` light `rgb(251,247,240)` | (251,247,240) | **(253,251,248)** | (253,251,248) |
| `--k-accent` light `#c67139` | (198,113,57) | **(228,178,130)** | (228,177,130) |

Three channels of three colours, to the unit. The encode is the ordinary sRGB
one, `1.055·c^(1/2.4) − 0.055`.

Alpha composites through the same wrong space and lands where the model
predicts: a dark key face, `rgba(255,255,255,.11)` over that `--glass`, measures
(123,119,114) against a predicted (120,115,109) — the residue is the key's own
border and shadow.

**Why it went unnoticed.** The error is nearly invisible where the design is
light — 251 becomes 253, and nobody measures a cream panel — and ruinous where
it is dark: a panel authored at 8% lightness is painted at 32%, so the dark
theme is not dark. This application's dark palette has looked "wrong, somehow"
for three days and was twice suspected of being mis-transcribed. It is correct
to the mockup's own numbers; the numbers just do not survive the trip.

**Not layer-specific.** Same binary, same sheets, in an ordinary window:
`rgb(22,17,12)` measures (81,74,65). Recorded here because this is where it was
found, not because the layer path owns it.

**Confirmed the panel really is opaque**, so this is not a compositing or
surface-alpha question: setting `--glass` to `rgb(255,0,0)` paints exactly
(255,0,0), which is the one value the sRGB curve fixes.

**Fix direction.** Decide where the conversion belongs and do it once. Either
linearise author colours when they are lowered — CSS colours are sRGB by
definition, so a renderer working in linear light has to convert on the way in —
or configure the surface as `*Unorm` rather than `*UnormSrgb` and keep the
values encoded end to end. The first is the usual choice, and it wants
consistency across solid fills, text brushes, shadows and gradient stops:
converting only fills would leave text lighter than its own background.

---

### 2-F. An application cannot name a font

`font-family: "Figtree"` — or any other name — resolves to nothing, because the
only faces the runtime has are three generics.

```
kit_bootstrap.rs:309-332   system_font_set()
    for generic in [SansSerif, Serif, Monospace] {
        let Some(face) = source.match_generic(generic) else { continue };
        set.push(FontFaceBytes::new(FontFamily::generic(generic), …));
    }
```

Three `fc-match` calls, three faces, each registered under a *generic* family.
`install_text_stack` adds one bundled last-resort face under `sans-serif`
(`:262-281`) and a per-character fallback (`:288-296`). Nothing anywhere loads a
face by name, and there is no API that would: `App` has no font method, and
`UiRuntime::set_font_resolver` hands over the whole port rather than adding to
it — an application that wanted one extra family would have to reimplement
resolution and then find a way to get the bytes to the renderer, which on the
layer path only `layer_runner` can do.

**What it looks like from the outside.** Exactly like 2-C did: a design that was
transcribed carelessly. This keyboard's sheets name `Figtree` and `Caprasimo`
throughout — they are the mockup's faces — and the interface draws in the
system sans. That is *partly* correct behaviour, since neither font is
installed here; what makes it a defect rather than a fallback is that installing
them would change nothing.

**The per-character fallback then picks by coverage alone**, which is the same
problem one level down and is what made it visible. `FontconfigFallback` asks
`match_char` and takes what it gets:

| character | `fc-match ":charset=…"` | drawn as |
|---|---|---|
| 😀 U+1F600 | DejaVu Sans | a hollow monochrome ring |
| 🥰 U+1F970 | Segoe UI Emoji | colour |

DejaVu Sans happens to carry a monochrome outline for `U+1F600`, so the query is
satisfied and the colour font is never reached. `U+1F970` is newer, DejaVu does
not have it, and the colour font wins by default. An emoji palette drawn this
way is mostly hollow rings with a scattering of colour — and the difference
between the two is which Unicode version added the character, which is not a
distinction any user can be expected to see.

**Fix directions.** Two, and they are independent. Resolve author-named families
through the same `fc-match` the generics already use, keyed by name, so a sheet
that asks for a font it can have gets it. And prefer a face whose *presentation*
matches the request when falling back per character — for an emoji-presentation
code point, a colour face over a text one — rather than taking the first font
that happens to have any glyph at that position.

---

### 2-G. ⚠️ WITHDRAWN IN PART — `overflow: auto` does scroll; the longhands do not exist

**Re-measured 2026-08-08. Scrolling works.** A container with `overflow: auto`
and more content than height accepts a wheel, records the offset, and the
display list changes:

```
block + overflow:auto   scroll_top after +120: 120.0
                        display list: 34 cmds before, 36 after, identical: FALSE
                        first difference at #1:
                          before  DrawText { origin: (0,0), … }
                          after   PushTransform(Affine [1,0,0,1,0,-120])
```

Identical for `display: grid` and `display: flex`. So the claim below — "the
display list is byte-identical before and after" — does not hold, and neither
does "whatever recognises a scroll container recognises the grid and not the
flex box": it recognises all three.

**How the measurement went wrong, twice.** The scroll is applied as a
`PushTransform` bracketing the subtree, not as an offset baked into each
command. Reading the `origin` of each `DrawText` therefore shows no movement
however far the container has scrolled — the origins are inside the transform.
Whoever repeats this must compare the *whole* command list, or look for the
transform. (Written from experience: the same wrong measurement was made twice
while checking this, before the command list was compared.)

**What is real, and it is the flex half of the original report.**
`overflow-y: auto` — the longhand — is not accepted:

```
block + overflow-y:auto   handle_scroll -> None    scroll_top -> None
```

Because `PropertyId` has **one** overflow property, the shorthand. There is no
`OverflowX` and no `OverflowY`, so `overflow-y: auto` resolves to no property at
all, the declaration is recorded as unsupported, and the box never becomes a
scroll container. The audit's flex case used `overflow-y`; that, and not the
display type, is why it answered `None`.

**Fix direction.** Add `overflow-x` / `overflow-y` as real longhands: two
`PropertyId` variants, `overflow` expanding to the pair through the existing
`expand_pair_shorthand` (which already does `[a] → both`, `[a, b] → first,
second`), and the three readers — `overflow_scrolls`,
`establishes_scroll_container`, `overflow_clips` — consulting the block-axis
longhand for scrollability and clipping when either axis clips. Note that
`ScrollOffsets` holds one axis, so `overflow-x: auto` can be *recognised* but
not scrolled; that limit should be reported rather than silently ignored, which
is now visible since 3-C.

**The emoji palette that prompted this** would have scrolled with
`overflow: auto` in place of `overflow-y: auto`.

The original report follows, kept because its "not investigated" question is now
answered: the offset *is* stored, and the paint *does* use it.

#### Original report

A grid with `overflow: auto` and more content than height accepts a wheel and
does not move.

```rust
let taken = runtime.handle_scroll(-120.0, Point::new(900.0, 300.0));
// taken == Some(NodeId(…))  — the grid is recognised as the scroll target
// the display list is byte-identical before and after
```

Tried at ±120 and ±400, in both directions, with a frame requested and run
between each. Every text run stays where it was. The same content in a
`display: flex; flex-wrap: wrap` box with `overflow-y: auto` is not even
accepted — `handle_scroll` answers `None` — so whatever recognises a scroll
container recognises the grid and not the flex box, and neither of them scrolls.

**Where it bites.** An emoji palette: 105 items in a panel that shows 88. The
design scrolls this; the consumer now sets `overflow: hidden` instead, on the
grounds that an `auto` which does not scroll is content placed out of reach
behind an interface claiming it can be reached.

**Not investigated:** whether a scroll *offset* is being stored and only the
paint ignores it, or whether nothing moves at all. `handle_scroll` returning the
node suggests the former.

---

### 2-H. Moving a layer surface repaints it

`LayerControl::reconfigure` is the only way to move a layer surface, and the
runner answers it with a full frame:

```
layer_runner.rs:161-165
    if let Some(pending) = layer_control.take_pending() {
        let _ = window.apply(&pending);
        runtime.request_frame(FrameReason::WindowChange);
    }
```

For a resize that is right. For a **margin** it is a repaint of content that has
not changed by one pixel — and a margin is what a drag is made of. A keyboard
carried across the screen re-renders itself once per position: fifty or sixty
whole frames per gesture, none of which draws anything new.

**What it costs.** Measured on this keyboard's own view at 1080×478, over
twenty frames each:

| | debug | release |
|---|---|---|
| build the view (the application's own work) | 147 µs | 68 µs |
| a whole frame — reconcile, style, lay out, paint | 10.0 ms | 0.93 ms |

The application's share is a rounding error either way. In release the frame is
under a millisecond and the repaint is affordable; in debug it is ten, and with
the GPU submission on top a drag runs at about a tenth of the rate the screen
could show — which reads as a broken gesture rather than a slow build, and cost
this consumer an afternoon of looking in the wrong place.

**Fix direction.** A reconfigure that changes only anchors, margins or the
exclusive zone needs a commit, not a repaint: `apply` can commit the surface
state and leave the frame alone, and the runner can request one only when
`LayerSettings` changed something that affects what is drawn — the size, the
scale, the display. The comparison is cheap and `LayerSettings` already derives
`PartialEq`.

---

### 2-L. A touch reaches the screen in a fifth of a second, and the frame is why

Measured end to end for the first time: from a tap leaving a virtual pointer to
the compositor reporting that pixel on screen. 120 taps, one scenario, on the
target.

```
touch -> pixels            p50  197.3   p90  439.4   p99  587.7   max  913.6 ms
  of which: to the handler p50   55.1   p90  174.3                max  294.8
  first repaint after it   p50   98.2   p90  286.6                max  442.6
frames built                                                      ~8 per second
frames actually shown      p50  316.7 ms apart                    ~3 per second
```

**The consumer is not in it.** A whole keypress handler costs `p50 0.12 ms,
max 1.23`; the mutex it takes, 0.00; the view it builds, 0.68. Everything above
happens after the application has finished.

**What a frame costs, per frame rather than averaged** (90 frames):

```
[frame] дерево 1.85 | style 4.14 | layout 7.96 | compose 4.75 | build 19.07   (p50)
        3.58        | 15.02      | 20.87       | 7.76         | 40.58         (p90)
        5.42        | 29.71      | 27.83       | 10.98        | 45.39         (p99)
```

Two things the mean was hiding. The tail: a `style` that averages four
milliseconds spends thirty in its worst frame, and a build that averages
nineteen spends forty-five. And the order: **`layout` now costs more than
`style`** — 7.96 against 4.14 at the median — which was invisible while both
were quoted as one average per sixty frames.

**More than half of what is built is never shown.** The compositor reports
about three presentations a second against eight frames built, and says so
explicitly: `frame N was never shown`. Frames are being produced that no one
sees, which is the cheapest work to stop doing and the hardest to notice without
`wp_presentation` — the phase counters call such a frame a success.

**Where the style time goes**, from `perf` on the target, symbolised:

```
4.76%  obscur_style::property::PropertyValue::uses_ch
3.97%  drop_in_place<PropertyValue>
3.97%  hashbrown::RawTable::reserve_rehash
3.57%  StyleResolver::resolve_property
3.17%  StyleResolver::resolve
2.78%  memcpy
```

Construction and destruction of `PropertyValue`, and a hash table growing — a
resolver rebuilding computed style rather than reusing it. That is a shape, not
a diagnosis; it is where a reader should start.

**How to reproduce this, exactly.** The numbers come from a harness in the
consumer's repository, and every one of them can be taken again:

```sh
perf/run-on-device.sh typing 15     # on the tablet
perf/report.py perfrun-<stamp>      # anywhere
```

It plays a fixed scenario through `zwlr_virtual_pointer_v1` — so the toolkit
sees a finger, timestamps and all — while sampling `/sys` at 10 Hz and recording
`perf`, then joins the tapper's log, the keyboard's trace, the toolkit's
per-frame lines and the presentation reports onto one timeline. It needs
`OBSCUR_FRAME_STATS=each` to get lines rather than means.

**A caveat that belongs with the numbers.** A short run is not a measurement:
four runs of the same build gave `style p50` of 4.83, 7.71 and 7.79 over
eighteen frames each. Anything under a hundred frames is inside its own spread.

**Measured on:** aarch64, Snapdragon 870, Adreno 650 (Turnip), 120 Hz panel,
`schedutil`, GPU at its 670 MHz ceiling. The frame budget at 120 Hz is 8.3 ms;
the median build is 19.

---

## Tier-3 — documentation defects

### 3-A. `key={…}` accepts a path and refuses a call, and says something else

`09-authoring-formats.md` makes a `<For>`'s `key` mandatory (§2.11) and does not
say what may appear in it. A method call is refused:

```
error: direct service access is not allowed in an expression (spec §2.10)
  --> src/…:38:44
   |
38 |     <For each={rows} as={row} key={row.keys.len()}>
   |                                    ^^^^^^^^^^^^^^
```

The rule is defensible — an identity that runs code is an identity that can
differ between two evaluations. The diagnostic is not: `.len()` on a `Vec` is
not service access by any reading a consumer has, and the message sends them
looking for a service they never wrote.

The practical consequence is worth a sentence in the doc, because it shapes the
model an application hands the markup: **everything a loop needs must already be
a field**. A keyboard's per-key class string, label and width are all derived,
so they are derived before the markup is reached and the view model is flat.
That is a good outcome; it is just not one the doc leads anyone to.

**Fix directions.** Say which expression forms `key` accepts; and either
re-word the diagnostic for the non-service case, or name the rule it is actually
enforcing.

### 3-B. A `<For>` body cannot splice a built node, and nothing says so

`{binding}` in body position interpolates **text**. There is no spelling that
puts an already-built `ViewNode` into markup, so a `<For>` whose body is a
complex element assembled by a function cannot be written — the function has to
become a component, or the loop has to move to Rust.

That is a reasonable design and it composes fine once components are available.
It is undocumented, and `09-authoring-formats.md:112`'s "Values reach it; shape
does not" is about the *file* route, so a reader on the `okui!` route does not
find it.

**Fix direction.** One line in the `<For>` paragraph: the body is markup, and a
node built in Rust reaches it as a component, not as a binding.

---

### 3-C. Nothing points an application at `set_diagnostic_sink`

The toolkit reports a great deal and the default sink is `NullDiagnosticSink`,
so an application that has not gone looking sees none of it. Everything in 2-C
was reported — thirty-two `css.unsupported_function` warnings, one per dropped
declaration, at boot — into a sink nobody had installed. Finding 1-B's silent
props mismatch is the same story from a different subsystem.

**It is not missing plumbing.** An application can install one today, and the
ordering works out:

```rust
let control = app.control();
control.with_runtime(|runtime| {
    runtime.set_diagnostic_sink(Box::new(StderrDiagnostics));
});
```

`AppParts::boot` queues `set_stylesheets` **last** (`app_parts.rs:96-103`) and
then applies the queue, so a sink queued before `run` is installed before the
sheets are compiled and catches their complaints. Verified: with the sink in
place, re-introducing one `background: var(--k-face)` produced
`css.unsupported_function unsupported function: background at 2486..2511` on
the next start.

The first thing it reported was a defect in the application nobody had noticed —
`tree.duplicate_key`, from two Shift keys and two spacer keys sharing an
identity within a row, rebuilding those rows every frame.

**Fix directions.** A line in the getting-started material would close most of
it. Better: `App::on_diagnostic(handler)`, so the useful thing does not require
knowing that `AppControl::with_runtime` exists and that the boot applies its
queue before compiling CSS. Best, and cheapest of the three: make the default
sink write warnings and errors to stderr in a debug build — a toolkit that
reports carefully into a sink nobody installs is indistinguishable from one that
does not report at all.

---

### 3-D. ✅ CLOSED — `[phase-stats]` called its total `frame` and stopped before the frame was shown

`OBSCUR_FRAME_STATS=1` is the toolkit's own answer to "where does a frame go",
and on the target machine it answers confidently and misleadingly:

```
[phase-stats] 60 frames | reconcile+build 0.88ms | style 20.53ms | layout 3.32ms | compose 3.56ms | other 0.00ms | frame 28.28ms
```

Read plainly, that says a frame costs 28 ms. What it measures is the runtime's
frame: the clock starts at `frame_pipeline.rs:78` and is stopped from
`compose_phase.rs:133`, both inside `run_due_frame_reactive`. Rasterising,
acquiring the swapchain image, blitting, submitting and presenting all happen
**after** the runner returns from that call (`layer_runner.rs:152-157`), and none
of them is in the number. Neither is the wait in `dispatch`, nor input
handling — `other` cannot hold them either, since it is derived from the same
total.

That is the whole of why 1-D took as long as it did: the instrument said the
frame cost 28 ms while the loop was in fact spending several hundred
milliseconds per turn, and the phase doing it was the one phase outside the
brackets. A counter that omits a phase is fair; one that omits a phase and calls
its subtotal `frame` sends the reader to look at `style`.

**Fix direction.** Name it for what it measures — `build 28.28ms` — and add the
two marks that close the loop: `present` around the renderer call and `waited`
around `dispatch`. Both belong to the runner rather than the runtime, which is
presumably why they were never added; a `FramePhaseClock` the runner can also
mark, or a second line printed by the runner on the same cadence, would close
it. Until then the honest reading of `frame` is "everything up to the point
where the GPU is asked to do anything".

---

## What this audit withdrew

**"A component tag inside a nested `<For>` may not work."** Reasoned, then
measured, and it is fine: `<For>` nests, a component tag inside one takes
expression props, and the ids agree. The keyboard's markup is

```rust
okui! {
    <div class="keyboard">
        <For each={rows} as={row} key={row.id}>
            <div class="row">
                <For each={row.keys} as={key} key={key.id}>
                    <Key key-id={key.id} label={key.label}
                         classes={key.classes} width={key.width}/>
                </For>
            </div>
        </For>
    </div>
}
```

and it lowers correctly — verified against the built tree, not by eye. The keys
do not appear on screen, and that is finding 1-A and nothing to do with this.

**"Text measurement ignores `letter-spacing`."** Reasoned from a clipped
`PRIVACY` badge — seven characters at 0.14em, and exactly one character missing
— and measured afterwards, where it does not hold: a run's `advance()` includes
the tracking (46.2px → 74.2px at 4px spacing) and the laid-out box grows with it
(48.2px → 76.2px). The clipping was 2-D wearing a different hat: the note beside
the badge was being wrapped against a ~200px available width, so the flex row
believed it was short of room and shrank the badge. `white-space: nowrap` on the
note fixed the badge, with its padding back to the plain `3px 8px` the design
asks for. Two symptoms, one defect.

**"The CSS drops are silent."** Written that way for a day, and wrong. The
compiler classifies every one of them (`UnsupportedCssFeature`), the pipeline
turns each into a diagnostic, and the runtime forwards it — to
`NullDiagnosticSink`, because this application had never installed one. The
defect is 3-C, which is a documentation defect and a two-line fix, not the
Tier-1 "the toolkit throws the evidence away" it was nearly written up as. The
lesson is the same one 1-A taught: check where the information goes before
concluding that it does not exist.

**"`ViewValue` should carry `f32`."** It carries `bool`, `i64`, `f64`, `String`
and `&str` (`view_value.rs:18-42`). A prop typed `f32` fails to compile with a
clear `From` error, and widening at the boundary is the caller's job. Correct as
it stands.
