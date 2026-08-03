# The input events get emitters

`docs/conformance-audit.md` recorded §2.3 as three-quarters built and unreachable:
the phase model existed, `EventPhase` / `EventDispatcher` / `EventPropagation`
had *"zero hits anywhere outside `obscur-interaction/src/event/`"*, and
`stop_propagation` / `prevent_default` were listed as equally unreachable from
production. By the time this campaign opened that entry was already stale — the
dispatcher had been wired — but the vocabulary above it had a different fault,
and a worse one for an author.

The reserved event set was six names. Four of them — `click`, `input`, `change`,
`longpress` — described things the runtime does. The other two, `keydown` and
`keyup`, had been added to both name mirrors and pinned in neither. Nothing
described the pointer at all. An author could write `on:pointermove` and it would
compile, lower to a symbol, hash cleanly, and never fire: a name is not an event
until something dispatches it.

**Status: closed for the pointer, the wheel, the crossings, the focus pair and
the context request.** Ten names were added, each with a production emitter
written before the name was reserved. The rule the campaign enforced on itself is
the one it existed to satisfy: *a name with no emitter must stay unreserved*, and
it is asserted in both directions, so `pointerover`, `pointerout`, `focusin`,
`focusout`, `scroll` and `menu` are all pinned as **not** reserved.

## The numbers

Measured on this tree, not copied from a stage report. The `HEAD` figure was
measured by stashing the working tree and re-running the suite.

| | |
|---|---|
| Suite | **6,453 passed / 0 failed** (6,392 at `HEAD` — **+61**, none lost, none weakened) |
| `clippy --workspace --all-targets` | **360** warning/error lines, **240** excluding per-crate summaries — unchanged across all five stages |
| `cargo fmt --all --check` | clean |
| `xtask check` (§3.16 architecture) | passes |
| `xtask consumer` (out-of-workspace facade consumer) | passes |
| Spec tests changed | **0** — no `spec_*` file appears in the diff, tracked or untracked |
| Reserved event names | **16**, dense from `0x0000_0101` to `0x0000_0110`, no holes |
| Names with a production emitter | **16 of 16** |
| New consumer test targets | 4 — `pointer_events.rs` (14), `focus_events.rs` (11), `wheel_events.rs` (7), `pointer_buttons.rs` (6) |
| Drift-guard tests | **20** in `okui_name_binding_drift.rs`, pinning all 16 spellings and all 16 constants |

## The vocabulary

Every reserved name, its emitter, and what refusing it suppresses. "Nothing" is a
result, not an omission — five of the sixteen have no default action to decline,
and each says so in its own `.doc` rather than being given an invented one.

| Event | Symbol | Emitter | `prevent_default` suppresses |
|---|---|---|---|
| `click` | `0x101` | `CommandDispatch::activation_command_for`, `command_dispatch.rs:56`, from `release_landed` | — it *is* a `pointerup`'s default action; refuse that instead |
| `input` | `0x102` | `EditableField`, `editable_field.rs:19`; `text_edit_commands.rs:109` | — |
| `change` | `0x103` | `text_editing.rs:112` (commit on blur); `command_dispatch.rs:64` (toggle/select activation) | — |
| `longpress` | `0x104` | `gestures.rs:106`, from the hold recognizer | — |
| `keydown` | `0x105` | `dispatch_raw_key`, `keyboard.rs:95` | the text edit, the shortcut, and the activation |
| `keyup` | `0x106` | `dispatch_raw_key`, `keyboard.rs:97` | **nothing** — `acts` is false for a release regardless |
| `pointerdown` | `0x107` | `dispatch_raw_pointer`, step 2b of `handle_pointer` | the focus move, the caret, the press anchor, gesture arming — and so any `click` that would have followed |
| `pointerup` | `0x108` | same site | the click, the `on:click`/`on:change` activation, the virtualized row click |
| `pointermove` | `0x109` | same site | the live drag-selection |
| `pointercancel` | `0x10A` | same site | **nothing** |
| `pointerenter` | `0x10B` | `dispatch_hover_crossings`, step 2a, from the hover transition | **nothing** |
| `pointerleave` | `0x10C` | same site | **nothing** |
| `wheel` | `0x10D` | `dispatch_wheel`, `scrolling.rs:154`, before the container moves | the container scroll, **and** the fall-through to `App::on_scroll` |
| `focus` | `0x10E` | `dispatch_focus_change`, `focus.rs:53` | **nothing** |
| `blur` | `0x10F` | `dispatch_focus_change`, `focus.rs:50` | **nothing** |
| `contextmenu` | `0x110` | `dispatch_context_menu`, `pointer_dispatch.rs:305`, from a secondary press | the focus move — which is the whole of what a secondary press performs |

Three names sit in `handle_pointer` in a fixed order, and the order is the
design. Step **2a** fires the crossings, **2b** the raw pointer event, **2c** the
context request; all three run before step 3 does anything with the press. That
is the DOM's out/leave → over/enter → move ordering, and it is what makes each
refusal above reachable — an event delivered after the runtime had already acted
could report the input but never decline it.

Within one event the crossings nest like a stack unwind: **every leave first,
innermost-first; then every enter, outermost-first.** A container is entered
before the row in it and left after it, so a teardown on leave is never
interleaved with the next node's setup.

## Where the names live

Three mirrors, and they agree — checked by reading all three, not by trusting the
guard:

- `crates/domain/obscur-view/src/view/event_name.rs` — the constants and
  `RESERVED_EVENTS`.
- `crates/domain/obscur-view/src/view/mod.rs` — the re-export; all 16 present.
- `crates/frontend/obscur-ui-ir/src/name_binding.rs` — the mirror, plus the
  `event_symbol` arm that lowers an authored `on:` spelling to the symbol.

`crates/facade/obscur-kit/tests/okui_name_binding_drift.rs` is what keeps them in
step, and it was widened from per-name assertions to whole-set ones. The two that
matter: `the_whole_reserved_event_set_is_the_same_set_on_both_sides` compares the
two slices wholesale, so a name added to one crate and not the other fails
without anyone extending the guard; `every_reserved_event_has_an_authored_spelling_that_reaches_it`
closes the other half — a symbol both crates agree on that no `on:` name lowers
to. That second test retro-covered `keydown` and `keyup`, which had been reserved
in both crates and pinned in neither.

`the_reserved_event_range_is_dense_from_one` pins the allocation rule: names are
taken in order from `0x0000_0101` and no hole is left behind. A hole is how two
people later pick the same number.

## The refusals

Named here because each one is a place where the obvious thing was not built, and
the reason is the record.

**No `prevent_default` was invented where there was no default action.** Five
names suppress nothing. `pointercancel` reports that the system destroyed the
sequence — honouring a refusal would reinstate the stuck `:active` and unreleased
capture that `cancel_landed` exists to clear. `pointerenter`/`pointerleave` are
dispatched after the chain is seated, and the restyle that follows is not the
crossing's default action, it *is* the crossing. `focus`/`blur` fire after focus
has already moved; the refusal that exists is on `pointerdown`, one event
earlier. Each is asserted in both directions — the same sequence run with
`refuse=false` and `refuse=true`, asserting the outcomes are identical — rather
than documented and left untested.

**A refused `pointerup` does not suppress the release bookkeeping.** Only the
click/activation block is gated. `PointerRelease::take`, `end_pointer`, capture
release and fling launch still run, because those are not actions to decline —
they are the fact that the finger lifted. Declining them would leave a stuck
`:active`, a capture nothing releases, and a pointer the runtime still believes
is down.

**A refused `pointermove` does not cancel a pan already scrolling.** The dispatch
sits *after* step 1b deliberately: the pan owns the pointer through capture
before the event is delivered, which is also why `target` is the capture target
by then. Refusing mid-scroll would leave the container half-scrolled with the
finger still down.

**`pointerover` / `pointerout` are not reserved.** A command routes either way
here, so the DOM's bubbling pair would be a second name for an identically
emitted event — the vocabulary-with-nothing-behind-it defect in a new costume.
Same reasoning for `focusin`/`focusout`: the focus binding already resolves
through the ancestor chain, so `focus` here *is* `focusin`.

**No `scroll` event.** `wheel` fires before the scroll and can refuse it; a name
meaning "it scrolled" would name a different event, and nothing emits that one.

**No `menu` event.** This toolkit opens no menu. `on:contextmenu` names the
request; drawing it stays the application's.

**No `PointerEventKind::Wheel`, and `WheelEvent` got its own payload variant.**
Reusing `EventPayload::Pointer` would have made a notch arrive as a `Move` that
never happened, with the delta riding in `PointerDelta` — "how far the pointer
travelled" — meaning something else for that one event.

**No programmatic focus setter.** The stage brief assumed one existed; it does
not, and that was verified rather than assumed — `self.focused` had exactly one
assignment site in the workspace. Building a setter means answering whether it
honours focusability and whether `FocusTree`/`TabOrder` finally get wired, which
is a feature, not an event name. What was built instead makes it cheap: the
transition was extracted into `move_focus_to`, the one door, so any future mover
emits `blur`/`focus` without remembering to.

**`WindowEvent::Focused` is still ignored**, with an explicit no-op arm carrying
the reasoning rather than falling into `_ => {}`. Routing a deactivated window
through the focus door would clear a field's caret and commit its `change`
because the user alt-tabbed. Doing it right needs a window-activation state and a
`:window-inactive`-shaped selector reading it.

## The cost gate

`pointermove` fires at the pointer's report rate, so the campaign's load-bearing
non-functional claim is that an application binding none of these names pays
nothing. Established by reading the code, in three parts:

1. `PointerEventBindings` (`pointer_event_bindings.rs`) is a single `u8`, seven
   bits, one per gated name. `binds` is a `const fn`: one `bit_for` comparison
   chain and one `AND` on a `Copy` field. No allocation, no walk.
2. The gate is checked **before** the resolve at all three sites —
   `pointer_dispatch.rs:329` (raw), `:351`/`:352` (crossings), `scrolling.rs:139`
   (wheel). The alternative is real: `CommandDispatch::binding_for` calls
   `AncestorChain::of`, which does `vec![target]` — a guaranteed heap allocation —
   plus an O(depth) walk with a `tree.get` per level. The walk is skipped, not
   merely empty.
3. The scan is recomputed only when `reconciled` is true (`frame_pipeline.rs:123`),
   one pass over live nodes with an early break once all seven bits are set, on a
   frame that already walks the tree several times. A binding can only appear or
   disappear on a commit that carried an operation, so that is the one moment the
   answer can change.

The gate is tree-wide rather than per-node, because a per-node answer would need
the very walk it avoids — the same trade `CommandDispatch::propagate` already
makes with `HandlerRegistry::has_listeners`.

`contextmenu` is deliberately **outside** the gate. It arrives at the rate a hand
can click, so the walk is once per right click, and a bit spent there would be
the eighth and last in the word. `focus`/`blur` are outside it for the same
reason: at most two walks per focus change, i.e. at most once per press.

This is reasoned, not profiled, and it is recorded as reasoned. The behavioural
half is pinned — `an_unbound_raw_name_is_never_dispatched` runs 64 moves for 0
dispatches while a bound `pointerdown` in the same tree still fires, and
`an_unbound_crossing_name_is_never_dispatched` gives 0 enters and 15 leaves
across 16 sweeps with only `pointerleave` bound.

## Pinned rules that changed

No `spec_*` file was touched anywhere in this campaign — verified against the
diff, tracked and untracked. Every changed assertion is a non-spec guard, each
carrying a dated note recording what it used to say:

- `name_binding.rs::the_raw_pointer_names_lower_to_their_reserved_symbols` — the
  negative loop lost `pointerenter`, `pointerleave`, `wheel` and `contextmenu` as
  each gained an emitter. **The rule did not change; only which side of it those
  names sit on.** Each moved to a positive assertion rather than simply being
  deleted, and `pointerover`/`pointerout` remain in the loop.
- `pointer_event_bindings.rs::the_four_names_occupy_four_distinct_bits` → six →
  `the_seven_names_occupy_seven_distinct_bits`. The rule — one bit per name, none
  shared — is unchanged, and the `OR` is what pins it; the count only stops a name
  being added without a bit.
- `pointer_event_bindings.rs::every_dispatchable_kind_has_a_name_and_the_crossings_do_not`
  → `..._and_the_crossing_kinds_route_elsewhere`. The assertions are unchanged
  (`name_for(Enter)` and `name_for(Leave)` are still `None`); the test's stated
  *reason* was wrong once the crossings had names. It now also asserts
  `bit_for` of both crossing names is non-zero, so "routes elsewhere" cannot
  decay into "is unreachable".
- `okui_name_binding_drift.rs` — the set-length pin went 12 → 13 → 15 → 16,
  each with a dated note, and every new spelling was added to the pairing loop
  above it so the count is never asserted alone.
- `pointer_converter.rs` — four tests had their *calls* updated for new
  parameters. No assertion was weakened; new ones were added beside them.
- `hover_update.rs` — `into_chain` became `into_parts`, so three tests read the
  chain differently and gained crossing assertions they did not have.

Across the whole diff, five assertion lines were removed. All five are the two
renames above; each has an equal-or-stronger replacement.

## Nothing regressed on the pointer

The gestures were the risk: the raw names dispatch on the same events the pan,
fling, long-press and capture recognizers read. Confirmed three ways.

Every documented threshold still matches its constant, and
`pointer_sessions.rs` — where all six live — is untouched by the diff:

| Gesture | Constant | Doc | Code |
|---|---|---|---|
| Pan starts after | `TOUCH_SLOP` | 10 px | `10.0` |
| Velocity sample rest | `PAN_REST_MICROS` | 100 ms | `100_000` |
| Fling decay | `FLING_TAU` | 0.325 s | `0.325` |
| Fling stops below | `FLING_REST_SPEED` | 20 px/s | `20.0` |
| Fling launches above | `FLING_MIN_LAUNCH_SPEED` | 50 px/s | `50.0` |
| Long press after | `LONG_PRESS_SECONDS` | 0.5 s | `0.5` |

The pointer test targets are **byte-identical to `HEAD`** — `git status` reports
`crates/application/obscur-runtime/tests/` clean — and all pass: `touch_gestures`
34, `scroll_containers` 11, `pointer_dispatch` 10, `text_mouse_caret` 10,
`click_commands` 8, plus `drag_and_drop` 7 and `spec_ch6_data_transfer` 4 in the
facade. 84 tests asserting exactly what they asserted before.

The one behaviour that *did* change on purpose is documented as a fix, not a
regression: a `PointerEventKind::Leave` now empties the hover chain instead of
hit-testing the position the pointer went out by. Delivering `CursorLeft` alone
would not have fixed the stuck `:hover` — `HoverUpdate::for_event` nulled the
target only for a touch contact that ended, so a mouse leave kept the row it was
over highlighted for the rest of the session.

## What still stops

- **`EventPhase` has 3 variants where §2.3 names 4.** Default action still has no
  variant, and adding one would name a stage `EventDispatcher` does not run — the
  default action is performed by `handle_pointer` / `handle_key` / `handle_scroll`
  *after* `propagate` returns.
- **`EventPropagation` has no production consumer.** `UiEvent` carries the
  propagation flags inline; the type is exercised only by `spec_ch2_interaction.rs`.
- ~~**The wheel is untrusted and unstamped.**~~ **CLOSED 2026-08-03.**
  `UiRuntime::handle_wheel(WheelEvent)` is the door and `PointerConverter::wheel`
  is the converter. `WheelEvent` gained `delta_x`, a `WheelDeltaMode`, a
  timestamp and trust; `handle_scroll(delta_y, position)` stayed as the shape
  every existing caller uses, meaning exactly what two floats can say.
  Two things the adapter had been destroying came back with it: the horizontal
  delta, dropped by a `_` pattern, and the line-versus-pixel distinction,
  flattened by a `WHEEL_LINE_STEP` that lived in the winit runner. The constant
  is `UiRuntime::WHEEL_LINE_STEP` now, because how far a notch scrolls is this
  toolkit's policy rather than a platform's fact.
  Still open beneath it: **nothing scrolls horizontally.** `ScrollOffsets` holds
  one axis, so `delta_x` reaches a handler and goes no further.
- **Only a pointer press moves focus.** No tab traversal, no programmatic focus.
  `TabOrder`, `FocusTree` and `SpatialNavigation` exist in the domain tier with no
  runtime caller — the only non-test use of `FocusTree` in the workspace is a spec
  test constructing one directly.
- **Focus is never reconciled against the tree.** A structural frame that removes
  the focused node leaves `focused` naming a dead id. It degrades quietly — the
  binding lookup finds no node and dispatches nothing — but the accessibility
  projection still publishes the stale id. Pre-existing, not introduced and not
  closed here; it is a focus-lifecycle defect rather than a naming one.
- **`MouseButton::Other(_)` is dropped**, rather than folded onto a modelled
  neighbour. Five buttons are delivered; a sixth is not invented.
- **The gesture arena is still bypassed.** The runtime resolves pan / long-press /
  fling through its own state, and a custom recognizer has no registration point
  on `UiRuntime`.
- **The `u8` is full.** Seven of eight bits are spent. An eighth gated name forces
  it to `u16`, which is a decision to take deliberately rather than discover.
