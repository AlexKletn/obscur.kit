# The debt page pays its own debts

`docs/api/12-reality.md` is the page that exists so nobody mistakes a type's
presence for a caller. It had gone stale about itself. Thirteen of its
thirty-three rows had moved and nobody had come back; three of the rows it
carried had already been decided — twice, in two documents, each pointing at the
other — and never carried out.

This campaign re-measured every row, carried out the deletions the page had
already argued for, refused the ones it had not, and closed two one-line debts
that were stated nowhere. It changed no behaviour. Every number below was
re-measured on the tree at the end, not copied from a stage report.

**Status: closed.** The page states what is true, the deletions are done, and
each refusal is recorded with the measurement that would have to change for it to
be revisited.

## The numbers

| | |
|---|---|
| Suite | **6,439 passed / 0 failed / 9 ignored** (6,481 when the campaign opened) |
| Tests removed | **43**, every one of them belonging to a deleted file |
| Tests added | **1** — the cost gate's eighth-bit alarm |
| Net | 6,481 − 43 + 1 = **6,439**, exact |
| `clippy --workspace --all-targets` | **360 warnings**, unchanged across all five stages |
| `cargo fmt --all --check` | clean |
| `xtask check` (§3.16 architecture) | passes |
| `xtask consumer` (out-of-workspace facade consumer) | passes |
| Files deleted | **125** — 14 `.rs` (1,302 lines) and 111 markdown across fourteen `.doc` directories |
| Source files whose logic changed | **0**. The only lines removed from a surviving `.rs` file are fourteen `mod` / `pub use` declarations of deleted types |
| `spec_*` files changed | **0** |

The falling test count is the point rather than a cost, and it is fully
attributable: `git diff` over the whole working tree removes exactly 43 `#[test]`
lines, and summing `#[test]` over the deleted files gives exactly 43. No
surviving test was removed, weakened, or reworded.

## What was deleted, and why

Four subjects, three rows. Each was verified as a closed island **before**
removal — grep across the whole workspace including `examples/`, `tests/`, the
facade barrel and the prelude — and each left its reason as a `// why` comment at
its own site, following the `StaleResult` precedent, because the code cannot
carry the reason once the code is gone.

### Vello `GlyphCache` / `PathCache` / `TextureCache`

40 files, 337 lines, 10 unit tests. **Unfillable by construction**, which is what
separated it from everything else considered. The claim that Vello's own atlases
make these redundant was checked against the dependency's source rather than
against the page: `vello_encoding-0.9.0/src/glyph_cache.rs` keeps a glyph cache
and prunes it by age, and `vello-0.9.0/src/lib.rs:333` holds the `image_atlas`.
Two of the three cached work this adapter never does and the dependency already
bounds. The third, `PathCache`, had no key at all — `obscur_scene::Path` holds
floats and so cannot derive `Hash`.

### `IntrinsicMeasure`, `MockIntrinsicMeasure`, `LayoutEnvironment`, `IntrinsicSize`

30 files, 328 lines, 11 unit tests. This was the oldest and worst of the three: a
documented architecture nobody built, and finding #1 of the conformance audit's
own headline list. Both leaf kinds already have a road and neither has the port's
shape — a text leaf is intercepted before any box algorithm is selected and
measured off seventeen inputs that `LayoutEnvironment`'s two fields cannot carry,
and a replaced element is sized *before the layout pass runs at all*, so no
measurement happens. Wiring it would have been a second spelling of
`ChildMeasurer`, and structurally impossible besides: algorithms address children
by index into `&[LayoutChild]`, the port by `NodeId`.

The debt was never really the trait. It was that **seven documentation claims
across three documents asserted the routing**, each quoting the last. All seven
are corrected.

### `StyleCache` and `LayoutCache` (with their `CacheEntry`s and `LayoutResult`)

55 files, 637 lines, 22 unit tests. Both roles were already occupied, and both
dead caches were keyed *worse* than the live ones, so wiring either would have
been a regression rather than a saving:

- `LayoutCache::get_arrange` keyed on the `NodeId` alone, where `ArrangeKey`
  carries five components precisely so a node that moved without its own
  constraints changing is not served a stale box.
- `LayoutCache` held one entry per node, so a node measured under two constraint
  sets in the same pass would evict itself; `MeasureCache` puts the constraints
  in the map key and keeps both.
- `StyleCache::get_versioned` compared a `VersionStamp`, and the only one
  production mints is `RetainedTree::version` — one counter for the whole tree —
  so every node would have missed together where the live per-node set misses
  one.

Both live mechanisms are pinned by tests that already existed:
`incremental_restyle.rs` asserts `style.resolved == 1` for a hover on one leaf of
a twelve-element tree, and `layout_coordinator.rs` asserts `arranged_count() == 0`
on an unchanged frame and exactly `3` for a dirty leaf.

## What was closed without deleting

- **`MouseButton::Other(_)`** — the drop is correct and is now a stated decision
  rather than a silent arm. The reason is not capacity, as was assumed:
  `PointerButtons` is a `u8` with three bits spare. It is meaning — the five
  masks *are* the DOM's `MouseEvent.buttons` bitmask, so `Other(code)` is a raw
  platform code with nothing in the vocabulary to become. The third reason, which
  nobody had written down, is that the drop is **symmetric**: press and release
  are refused identically, so the held-button set stays honest. The existing test
  was widened from one sampled code to nine, so the pinned claim is "none folds"
  rather than "`Other(9)` did not".
- **The pointer cost gate's `u8`** — not widened, because no eighth gated name
  exists and widening ahead of one is a surface with no caller. Instead the
  landmine announces itself: a new test fails on the **eighth** name rather than
  the ninth, so the commit that spends the last bit is the one that takes the
  decision. It was verified by breaking the constant and watching it fire, not by
  assuming. It also documents the silent-failure mode that made the gate worth an
  alarm: `bit_for` answers `0` for an unknown name, `binds` reads `0` as "not
  bound", so a gated name added with nowhere to go fails **closed and silent**.
- **Three stale claims in `docs/input-event-model.md`** — Tab traversal, focus
  reconciliation against the tree, and the programmatic focus setter were all
  described as missing. All three had shipped one commit *before* that page was
  last written. Five line citations had also drifted, one of them onto a real but
  unrelated function, which is the expensive kind: a reader following it lands
  somewhere plausible and never suspects the citation moved.

## What was refused

Refusals are results. Each is recorded with what would have to be true to
revisit it.

| Refused | Why, and what would change it |
|---|---|
| `GamepadEvent` / `Button` / `Axis` / `Id` | Zero callers is true, but it is a **spec-assigned responsibility** of `obscur-interaction`, not scaffolding, and unlike the Vello caches it is *fillable* — what is missing is an adapter, not a producer that cannot exist. Revisit if the catalog decides gamepad input is out of scope |
| `HapticRequest`, `HapticPattern` | §6.16 lists `haptics` beside the gesture arena, and §6.16 is the named next step of the touch work already underway. Deleting the model of an approved next feature is not debt closure |
| `FocusTree`, `FocusResult`, `GestureArena`, `GestureContender`, `ReplaySession` | Each is constructed by a `spec_*` test. Deleting them means deleting an oracle, which this campaign does not do |
| `FocusRestore`, `SpatialNavigation`, `NavigationDirection` | The strongest zero-caller signal after the Vello caches — literally no reference outside their own files. Kept because they sit in modules whose siblings are live (`FocusScope` 20 production references, `FocusRequest` 10, `TabOrder` 4, `SpatialIndex` 3): an unfilled slot in a working module, not a module that cannot work |
| `InjectionSink` | **Filed under the wrong heading, and the filing was the error.** `Endpoint::new` constructs one on every endpoint (`endpoint.rs:81`) — that *is* the fail-closed refusal. Removing it would remove the default that keeps injection refused |
| `EventPropagation`, `EventPhase`'s missing fourth variant | Driven by `spec_ch2_interaction.rs`; and wiring means replacing `UiEvent`'s inline flags on the live path — a refactor with behavioural risk across every dispatch site, not a one-liner |
| `HitTest`, `HitChain`, and `obscur-layout`'s `pass/` island | Found during the campaign, absent from the page, genuinely zero-caller. **Not deleted, because no stage owned the row** — a deletion decided in passing is one nobody can audit later. Recorded on the page instead, with the measurement, so the next decision is cheap |
| Widening the cost gate to `u16` | No eighth name exists. The alarm is the answer that does not pre-commit a decision with real content |
| Editing `specs/…/03-source-layout-by-crate.md` | Three spec source trees still draw files that no longer exist. That divergence is recorded on the page rather than hidden, and the spec entries are now the mistake — but editing the spec is not this campaign's authority |

## What the campaign learned about finding this class of debt

Four tests emerged, each from a case the one before it could not settle. They are
recorded because the next reader will need them and because each was earned:

1. **Is it fillable?** The Vello caches were not — the dependency owns the work
   and already bounds it. The gamepad vocabulary is. Spec-listedness cannot be
   the deciding test on its own, because the deleted caches were spec-listed too.
2. **What would the caller have to pass?** Zero callers said `IntrinsicMeasure`
   was unused; the seventeen-argument call it would have had to serve said it was
   *unusable*. That is a better deletion test than any reference count.
3. **Is the role already occupied — and if so, diff the keys.** Both dead caches
   were perfectly fillable and their signatures were unobjectionable. What
   settled them was that comparing the keys side by side turns "redundant" into
   "would be wrong". A dead cache is a preserved earlier answer, and the shipped
   answer is usually later and better.
4. **When the brief hands you the reason, measure the reason.** The
   `MouseButton::Other` brief offered "`PointerButtons` is a five-bit set with no
   room". It has three bits spare. Accepting the offered reason would have
   produced a `.doc` stating something false.

And one limit that outranks all four, because it is what let this debt accumulate
and then let two passes of re-measurement each leave something wrong:

> **A grep that confirms cannot enumerate.** It finds the symbol someone already
> suspected and stops. The border-box clamp row was widened from "flex" to "flex
> and grid" and was *still* too narrow — enumerating `\.constrain(` finds four
> algorithms. The `ch` row named three of four production supply sites. The
> gamepad row counted a neighbouring file's tests as its own. `HitTest`,
> `HitChain` and a four-type island in `obscur-layout/src/pass/` were missing
> from the page entirely, for as long as nobody happened to suspect them.

The mechanical fix is a cross-crate reachability pass in `xtask`, which would find
all of them without a human guessing first. Two things make the case for it:
`LayoutResult` was reachable only from dead code and passed every check the
toolchain has — it is `pub`, re-exported, compiling, its tests green — and `xtask
check` and `xtask consumer` both print *"architecture checks passed"* before and
after deleting fourteen files that the spec's own source trees draw. Nothing in
the toolchain enforces those trees in either direction.

## Two smaller notes for whoever is next

- **A one-line `_module.md` under a `.doc/` directory is a tell** that a tier was
  materialised rather than designed, and it is greppable. The counter-example is
  worth knowing too: the Vello `cache/mod.doc/_module.md` was forty lines of
  correct, load-bearing analysis that had already reached the right verdict and
  stopped at *"the decision is the owner's and is unmade"*. That is the shape of
  debt this campaign mostly found — not unanalysed code, but analysed code nobody
  closed the loop on.
- **`okui_screen_state.rs:251` is intermittently flaky.** It asserts a wall-clock
  *ratio* between two measured times. It failed once across the campaign's seven
  full-suite runs and passed the other six. It was reported and deliberately not
  touched: a ratio assertion between two measured times is a pinned rule someone
  chose. It remains capable of turning a clean run red.
