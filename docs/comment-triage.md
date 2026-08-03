# Comment triage

**A comment that retells the code is deleted; a comment that says WHY is kept;
and where a comment exists only because the code is opaque, the code is fixed and
the prose goes away.** Spec §3.1 already says a file holds one entity and names it
honestly — a comment that has to introduce a block is that law broken quietly.

Bias: this repository's comments are unusually high-value — they are its memory
of decisions. **When unsure, keep.** Leaving a narration behind is cheap;
destroying the record of a refusal is not.

**Status: closed.** Eleven stages read every non-doc `//` line in `crates/` and
`tools/` — 15,277 of them across 809 files — and deleted 425. That is **2.8%**,
and the small number is the campaign's main result, not a shortfall: the areas
that yielded least are the ones that hold the most knowledge.

Measured on the tree as it stands, not copied from a stage report:

| | before | after |
|---|---|---|
| non-doc `//` lines in `crates/` + `tools/` | 15,277 | **14,852** |
| files carrying one | 809 | 803 |
| suite | 6,377 / 0 / 9 ignored | 6,377 / 0 / 9 ignored |
| `clippy --workspace --all-targets` | 360 | 360 |
| `fmt`, `xtask check`, `xtask consumer` | clean | clean |

Not one test was added, removed, weakened or reworded, and no assertion changed:
`git diff` over the whole campaign contains no `#[test]` line and no `assert` line
on either side. 30 private functions and 4 consts were named into existence, and
about two dozen bindings renamed, because a comment existed only to say what they
were.

## The rubric

### DELETE

- restates the line or expression under it (`// sort the files` above `files.sort()`);
- narrates an obvious step inside a short function;
- repeats what the item's own `#[doc = include_str!("<file>.doc/*.md")]` already
  says — **this was the highest-yield rule in the repository**, see below;
- commented-out code;
- a section banner labelling something the file's own entities already name.

### KEEP — never delete

- any `§x.y` spec citation, or a reference to `specs/w3c/*`;
- a dated note (WEAKENED / RESTORED / DELETED / NOTE / CLOSED yyyy-mm-dd);
- a refusal, a rejected alternative, "the bug this pins", "this used to …";
- a warning against a plausible wrong change ("do not X, because Y");
- a platform, browser or spec quirk, or a measured number;
- a TODO or a named known limitation;
- in a test, the reason the assertion is what it is.

### CHANGE THE CODE

Behaviour-preserving only: no signature changes, no reordering, nothing that can
alter what runs. Rename the badly-named binding, lift the unexplained magic
number to a named `const`, extract a named private helper for the block the
comment had to introduce. Then delete the comment, because the code now says it.

### Three rules the campaign added

**1. Read the `.doc/` file first.** This repo externalised every doc comment, so
an inline `//` retelling a numbered flow the markdown two directories away already
spells is easy to write and easy to miss. In four of the eleven areas this rule
alone accounted for most of the deletions — 22 of 37 in `runtime/`, 20 of 33 in
the domain tail, 13 of 22 in `obscur-layout`. `keyboard.doc/handle_key.md` spells
steps 1, 2, 3, 3b and 4 of `handle_key`, and the function narrated all five a
second time. `timeline.doc/advance.md` is an exhaustive prose account of the
integrator, and `Timeline::advance` re-narrated it one branch at a time — six
comments in one function, the largest single reduction in the campaign. The
discipline is: open the `.md`, read it in full, then delete. Never assume.

**2. A coherent set is kept whole, or not at all.** Several banner runs mix
content-free labels with §-citations and real claims. Deleting the two members
that restate their code, and leaving the rest, reads worse than leaving all of
them: the table stops being a table. Precedent set at
`xtask/src/rule/dependency_graph/layer.rs` in pass 1 and applied afterwards to
`pointer_sessions.rs`, `handler_registry.rs`, `plural_rules.rs`,
`syntax_kind.rs`, `relative_units_paint.rs` and `builtin_components.rs`.

**3. Deleting a comment can change what clippy sees.** `clippy::collapsible_if`
is suppressed by the presence of a comment inside the outer block, so removing
one un-suppresses the lint. It happened once, in
`obscur-resources/src/hub/resource_hub.rs`, and was closed by collapsing the
nested `if let` into a let-chain — behaviour-identical. Re-count workspace
warnings after a pass, not only tests.

## Per area, measured

| area | before | after | deleted | code changed |
|---|---|---|---|---|
| `foundation` + `diagnostics` + `tools` + `components` | 707 | 684 | 23 | 4 |
| `domain/obscur-layout` | 1,457 | 1,435 | 22 | 5 |
| `domain/obscur-style` + `domain/obscur-text` | 1,450 | 1,373 | 77 | 7 |
| `domain` (the other fourteen crates) | 824 | 791 | 33 | 6 |
| `runtime/src/orchestration` | 1,842 | 1,817 | 25 | 6 |
| `runtime/src/runtime` | 1,654 | 1,617 | 37 | 6 |
| `application` (rest: frame, transaction, headless, testkit, tests) | 1,017 | 999 | 18 | 2 |
| `frontend` (css, okui, ui-ir, ui-macros) | 1,380 | 1,322 | 58 | 6 |
| `adapters` | 1,145 | 1,122 | 23 | 4 |
| `facade/obscur-kit/src` + `examples` | 1,431 | 1,331 | 100 | 1 |
| `facade/obscur-kit/tests` | 2,351 | 2,342 | 9 | 1 |

Both columns were re-measured at close — the "after" against the worktree, the
"before" against `HEAD` — and **all twenty-two figures match their stage reports
exactly**. The eleven areas partition the workspace: their before-column sums to
15,258, which is `HEAD`'s 15,277 minus the 19 lines in
`crates/facade/obscur-kit/build.rs`, the only commented file no area covered.

One number circulated during the campaign was wrong, and it was not a stage's.
The opening total given to the stages, 15,193 across 808 files, is 84 lines and
one file low; `git` says `HEAD` carries **15,277 across 809**. The diff settles
it independently: 494 non-doc `//` lines removed, 69 added, net **−425**, which
is exactly what the eleven reports sum to.

The largest deletion counts are in the two youngest, most mechanical layers —
the CSS parser/lowering pass and the kit's example scenes — and the smallest are
in the spec test suite (9 of 2,351) and the layout algorithms (22 of 1,457).

## Where the code changed because a comment existed only to explain it

This is what the campaign was for. 30 private functions, 4 consts, ~24 renames.

### Named helpers extracted (the function name is the deleted sentence)

| new item | file | the comment it retired |
|---|---|---|
| `pop_line_terminator` | `obscur-okui-lsp/transport/reader.rs` | `// Strip the trailing CR/LF.` |
| `Select::open_option_list` | `obscur-widgets/select/select.rs` | the two-line description of an 18-line block inside `contents` |
| `next_unused_compatible` | `obscur-tree/reconciliation/child_matcher.rs` | `// Forward scan for the next unused compatible unkeyed child.` — all four words are now in the name and the `from` parameter |
| `RetainedTree::reuse_vacated_slot` | `obscur-tree/tree/retained_tree.rs` | `// Reuse a vacated slot, advancing its generation on reuse.` The sibling arm was already `push_fresh_slot`; now both are named calls and `alloc` is five lines |
| `is_utf8_continuation` | `obscur-localization/message/message_pattern.rs` | `// Walk to the end of this UTF-8 sequence so the slice is valid.` The mask is now spelled in binary, where `10xxxxxx` is visible |
| `last_keyframe_at_or_before` | `obscur-motion/track/keyframe_track.rs`, `runtime/orchestration/motion_coordinator.rs` | the same hand-rolled `while i + 1 < frames.len()` scan, and the same comment, in two crates |
| `zero_trailing_hang_run`, `trim_trailing_tracking` | `obscur-text/layout/packed_line.rs` | `// 1. Zero the trailing collapsible-whitespace run.` / `// 2. Trim the trailing letter-/word-spacing…` — `hang_trailing` is now two calls |
| `next_line_start` | `obscur-text/layout/paragraph_layout.rs` | `// The line's byte range runs from its first glyph's cluster to the next line's first glyph cluster (or the text end).` |
| `line_index_at` | `obscur-text/layout/text_hit_test.rs` | `// Choose the line by vertical position, clamping to first/last.` |
| `leading_whitespace_len`, `leading_word_len`, `trailing_whitespace_len`, `trailing_word_len` | `obscur-text/editing/text_buffer.rs` | four `// Phase 1:` / `// Phase 2:` banners. `next_word` and `prev_word` are now three lines each and read as their own description |
| `all_sides_solid` | `runtime/orchestration/border_paint.rs` | `// Every side solid.` (its sibling `// All four widths equal and positive.` was deleted outright — the expression is the sentence) |
| `nothing_to_collapse`, `normalized_segment_break` | `runtime/orchestration/text_attributes.rs` | a six-clause boolean guard, and an inline peek/next dance. Half the first comment survives, naming the CSS modes the branch serves, which the code does not |
| `resolved_corner_radii` | `runtime/orchestration/scene_coordinator.rs` | the naming sentence of a 12-line block inside the 480-line compose walk. The §4.1/§4.5 rationale moved onto the helper intact |
| `spans_multiple_scalars` | `runtime/key_edit_resolution.rs` | `// A single-scalar cluster: defer to the buffer's default delete.` The *forward* branch had the identical opaque expression and no comment at all; it is now equally readable |
| `ReconcileEnactor::commit_if_changed` | `runtime/transaction/reconcile_enactor.rs` | two comments over an identical eight-line tail duplicated in `commit_root` and `commit_subtree`. "On the same terms as a root commit" is now enforced by there being one body |
| `tree_payload`, `layout_payload`, `styles_payload`, `focus_payload`, `accessibility_payload`, `scene_payload`, `animations_payload`, `resources_payload`, `caches_payload` | `obscur-kit/src/devtools.rs` | `inspect_source_from` was ~150 lines of nine consecutive blocks, each introduced by a `// Tree:` / `// Layout:` / `// Focus:` label — the exact shape §3.1 forbids. It is now a fifteen-line composition |

### Magic numbers lifted to a name

- `TEXT_DOCUMENT_SYNC_FULL: i64 = 1` — `obscur-okui-lsp/server/server.rs`, from
  `// 1 = full-text document sync.`
- `PLACEHOLDER_PART_ID_SPACE: u64 = 1 << 61`, `SCROLLBAR_PART_ID_SPACE: u64 = 1 << 62`
  — `runtime/tree_inspector.rs`, from six lines decoding two bare shifts. The
  invariant they carried (real ids never reach these bits; the two spaces are
  different bits) went into two new `.doc/*.md` files, because a `const` cannot
  state it.
- `VSYNC_TICKS_IN_TEN_SECONDS: usize = 625` — `tests/spec_ch3_scheduling.rs`,
  from `// 625 * 16 ms = 10 000 ms.`

### Renames (the badly-named binding was the whole reason for the prose)

`y` → `last_real_child_bottom`, `b` → `last_real_child_block_end`, `pending` →
`open_margins`, `gap_acc` → `accrued_gap` (both margin-collapsing loops in
`flow_layout.rs`, where a five-line comment existed only to translate three
names — what remains are the three lines carrying CSS 2.1 §8.3.1 and the
never-collapses rule); `norm` → `from_end_line_to_positive_line` and `track` →
`zero_based_track_of_line` (`grid_placement.rs`); an extracted `is_growable`
predicate spelled twice inline (`grid_track_sizing.rs`); `lines_glyphs` →
`packed_lines` (`paragraph_layout.rs`, `text_layout.rs` — it held `PackedLine`s,
not glyph vectors); `bi` → `next_break`, `best_end` → `widest_fitting_end`,
`first_end` → `first_cluster_end`, `acc` → `head_width` (`paragraph_word.rs`);
`a`/`b` → `snapped_start`/`snapped_end` (`text_buffer.rs`); `keys` →
`properties_in_either`, `names` → `names_in_either` (`style_diff.rs`);
`projected` → `projected_child_count` (`tree_transaction.rs`); `seen` →
`seen_kind_and_target` (`transaction_validator.rs`); `w`/`h` →
`scrollport_width`/`scrollport_height` (`caret_scroll.rs`, whose comments had to
spell them out as `W` and `H` and then re-reference them three times each);
`offset` → `content_space_offset` (`virtual_lists.rs`); `cap` →
`ancestor_walk_limit` at three cycle-guard walk sites (`hit_testing.rs`,
`scroll_offsets.rs`); `entries` → `contract_entries` (`contract_derive.rs`, whose
sibling was already `surface_entries`); `unit`/`non_negative` →
`clamp_to_unit_range`/`clamp_to_non_negative` (`filter_lowering.rs`); `running` →
`highest_position_so_far` and `highest_magnitude_so_far` (`gradient_lowering.rs`
— `p.max(running)` says nothing about monotonicity); `groups` →
`argument_groups` (`transform_lowering.rs`, 11 sites); `temp_a`/`temp_b` →
`horizontally_blurred`/`fully_blurred` and `src` → `copied_region`
(`gaussian_blur.rs`, retiring four ping-pong narrations); `scratch` →
`color_matrix_scratch` plus three shadowed inner names
(`filter_layer_pass.rs` — the shadowing was half the opacity).

### One correction, not a deletion

`obscur-render-cpu/renderer/cpu_renderer.rs` carried two consecutive paragraphs
describing the same `layer_stack`, and the **first was stale**: it said the entry
holds an opacity, when the entry has held `(opacity, blend_mode)` since blend
modes landed. Merged into one accurate paragraph that keeps the balance-invariant
sentence the newer paragraph had dropped.

### One latent defect found by looking at a comment

In `runtime/orchestration/arrange_pass.rs` the comment beginning *"Recurse into
in-flow children…"* no longer sat on the loop it described — a `scrollport`
binding with its own comment had been inserted between them, so two unrelated
paragraphs had run together. The narration half went; the surviving warning
(*"Zipping bounds both sides, so a mismatched length never indexes out of
range"*) is now attached to the `for` loop it guards.

## Areas that honestly needed little, and why

- **The spec test suite** (`obscur-kit/tests`, 9 deleted of 2,351 — 0.4%). This
  is the oracle. Its prose is overwhelmingly the reason an assertion is what it
  is, a §x.y or `specs/w3c` citation, a dated `WEAKENED`/`CORRECTED` record, or a
  measured trap. A scan for a `//` block sitting directly above a
  `#[doc = include_str!]` attribute found **zero** instances across its 128
  files: the externalisation pass had already done its job. `builtin_components.rs`,
  the largest file at 173 comment lines, lost nothing.
- **`obscur-layout`** (22 of 1,457). The three big algorithm files hold 1,033 of
  those lines and lost two, both replaced by binding names.
  `resolve_flexible_lengths.rs` lost none: its step comments are css-flexbox-1
  §9.7 sub-steps, and its two longest paragraphs record the NaN freeze-loop hang
  that took the event loop and the window's close button with it.
- **`runtime/src/runtime`** (37 of 1,654, 2.2%). This module is the runtime's
  decision log. Whole files — `frame_plan.rs`, `controlled_reseed.rs`,
  `compose_only_edit.rs`, `cascade_sheets.rs`, `background_work.rs`,
  `gesture_arming.rs`, `accessibility_publisher.rs` — yielded nothing at all.
- **`crates/adapters`** (23 of 1,145). This layer barely narrates: it is winit,
  Wayland, wgpu, Vello, tiny-skia and HarfRust quirks, and the pixel probes that
  pin them. `winit_window_backend.rs` lost none of its 66.
- **`obscur-style/computed/initial_value.rs`** (154 comments) and
  **`property/property_id.rs`** (72): read line by line, untouched. They are
  policy tables of refusals — *"left unresolved because …"*, *"byte-for-byte
  identical to before X existed"* — and the file-level note that arms sharing a
  body are deliberately not merged is what justifies the
  `#[allow(clippy::match_same_arms)]` beneath it.
- **`obscur-ui-ir/name_binding.rs`** (125 comments, zero deleted). Every match
  arm carries a refusal (*"`textarea` is a third spelling of the one control"*),
  a regression record, or a §2.12/§4.16 citation. A "simplification" here that
  collapsed `"Input" | "input" | "textarea"` would silently break a front end,
  and the comment is the only place that danger is written down.

## The damage audit

Every deleted `//` line in the campaign — all 494 of them — was re-read at close
against the KEEP rules, and every hit for `§`, `specs/w3c`, a dated note, a
refusal, *"because"*, *"never"*, *"deliberately"*, *"used to"* was traced to
where it went.

**Nothing was restored, because nothing was lost.** Each of the seven §-carrying
and quirk-carrying lines that the diff removes was verified to survive:

- the §4.1/§4.5 corner-radius paragraph moved verbatim onto `resolved_corner_radii`;
- CSS 2.1 §8.3.1 survives in the rewritten `flow_layout.rs` lead comment;
- the §5.19 budgeted-cache note and the `EventFlow`/`PluginState` refusal moved
  onto `caches_payload` and stayed on `inspect_source_from` respectively;
- the "allocation-light, dependency-free" rejected alternative survives in
  `transaction_validator.rs`;
- `line_break.rs`'s hard-break suppression rationale was a duplicate of the
  surviving use-site comment eight lines below it;
- the premultiply rationale in `filter_layer_pass.rs` and the definite-height
  note in `caret_scroll.rs` are both present, reworded only where a renamed
  binding made the old wording false.

Each `.doc/*.md` claimed as the reason for a deletion was opened and checked:
`handle_key.md`, `advance.md`, `apply.md`, `compute.md`, `find_next.md`,
`extract.md`, `sticky_offset.md`, `resolve_absolute_box.md`, `resolve_axis.md`,
`UiIrValidator.md`, `commit_if_changed.md`, and the two new part-id-space docs.
All of them say what was claimed, and most say more.

**One regression was found and fixed here, and it was not a comment.** The last
stage's `VSYNC_TICKS_IN_TEN_SECONDS` const sat mid-function, which tripped
`clippy::items_after_statements` and moved the workspace count from 360 to 361.
The pattern is the suite's own idiom — ten other instances carry the same warning
— but a comments campaign has no business moving a measured baseline, so the
const was hoisted to the top of its function. A const's position in a block does
not change what runs. Clippy is back at **360**.

## Left open, deliberately

- **`grid_track_lowering.rs:570`** — `// ---- the flex shorthand (CSS Flexbox 1
  §7.1) ----` is the last line of the test module and labels nothing; the
  flex-shorthand tests moved to `shorthand_expansion.rs`. It was kept only
  because the hard rule forbids deleting a §x.y citation, and no stage was
  willing to set the precedent of judging one. It is dead and should go, by a
  decision rather than by a sweep.
- **`frame_phase_clock.rs`** keeps its per-phase totals in a thread-local
  six-tuple, so `finish` reads `t.0` … `t.5` and a comment has to say which of
  five untracked things `other` covers. Naming those fields is a real
  improvement, but it introduces a second entity into the file and rewrites
  `PHASE_TOTALS.md` — its own change, with its own review.
- **`grid_area_template.rs`** accumulates into
  `Vec<(Box<str>, usize, usize, usize, usize, usize)>` whose comment exists purely
  to decode the anonymous 6-tuple. The honest fix is a named private struct; it
  would need new `.doc/*.md` files, so only the adjacent pure restatement went.
- **`spec_ch5_editing_ime.rs`** explains the binding `k` — textbook "rename the
  badly-named binding" — and it was refused: `k` and `n` appear inside four
  assertion *message strings*, so the rename cannot be done without editing
  assertions, and assertions are never edited.

## Worked examples (pass 1: foundation / diagnostics / tools / components)

### 1. A helper named after the comment — `obscur-okui-lsp/src/transport/reader.rs`

Before:

```rust
        // Strip the trailing CR/LF.
        while matches!(line.last(), Some(b'\n' | b'\r')) {
            line.pop();
        }
```

After — the loop moved to a private fn whose name is the deleted sentence:

```rust
fn pop_line_terminator(line: &mut Vec<u8>) {
    while matches!(line.last(), Some(b'\n' | b'\r')) {
        line.pop();
    }
}
…
        pop_line_terminator(&mut line);
```

### 2. A magic number lifted to a const — `tools/obscur-okui-lsp/src/server/server.rs`

Before:

```rust
                // 1 = full-text document sync.
                ("textDocumentSync", JsonValue::int(1)),
```

After — the `1` is now spelled, so nothing has to be said about it:

```rust
const TEXT_DOCUMENT_SYNC_FULL: i64 = 1;
…
                ("textDocumentSync", JsonValue::int(TEXT_DOCUMENT_SYNC_FULL)),
```

### 3. An opaque path walk named — `tools/xtask/src/rule/module_layout.rs`

`// <crate>/src/lib.rs → <crate>/Cargo.toml` existed because `.parent()
.and_then(Path::parent)` says nothing. Naming the result says it instead:

```rust
    let Some(crate_root) = file.parent().and_then(Path::parent) else {
        return false;
    };
    fs::read_to_string(crate_root.join("Cargo.toml"))
```

### 4. Banners — `wire_codec.rs`, `endpoint.rs`

Seven `// --- Handshake -------------` rules deleted: `push_handshake_request`,
`push_command`, `push_snapshot` already say which section they are in. Likewise
`// Handshake.` / `// Subscribe.` / `// Inspect.` inside a test already called
`end_to_end_handshake_subscribe_inspect_and_audit`.

## Kept, and the rule that kept it

- **`corner_radii.rs` (76 comments, none deleted).** Every one is a §4.1 / §4.2 /
  §6.1.1 / §8.3 citation, a number the spec supplies (`27.5`, `40/140`), or the
  reason an assertion is written the way it is — including `// The guard on the
  fix that would have been wrong.`
- **`xtask/src/rule/dependency_direction.rs`** — `NOTE: taffy is deliberately NOT
  here …`. A refusal and a rejected alternative.
- **`obscur-task/src/task/mod.rs`** — the paragraph explaining that `StaleResult`
  was deleted rather than given an invented contract. The code cannot contain
  this, because the code is its absence.
- **`xtask/src/rule/dependency_graph/layer.rs`** — the nine
  `// domain -> foundation + approved lower domain.` arrows. Two of them restate
  their `matches!`, but most carry a parenthetical the code does not
  (`(+ sibling frontend crates)`, `application (headless)`, `frontend (UI IR)`).
  Kept as a set: breaking a coherent policy table for two lines reads worse.
- **`select_component.rs`, `input_props.rs`, `button_props.rs`** — the
  "idempotence precondition" paragraphs: why mounting a component on a
  builder-made node is a no-op. Nothing in the code says that.
- **`obscur-system/src/power/power_provider.rs`** — *"Rewritten 2026-07-20
  (owner-approved): the §5.18-style 'return the plugged-in baseline' decision was
  retracted for power."* Three keep rules at once, and the code cannot hold it,
  because what it records is a decision that was reversed — and an absence leaves
  no trace.
- **`obscur-routing/src/router/router.rs`** — the same two-line comment in both
  `back()` and `forward()`, explaining that the borrowed entry is dropped at the
  semicolon so `resolve_current` can re-borrow `self`. Duplicated prose is
  normally a smell; this one guards a `?;` that discards its value on purpose.
  Binding the entry is the obvious "improvement" and it does not compile. Kept in
  both places.
- **`obscur-kit/src/app/runner.rs`** (108 comments, none deleted) — including
  *"`advance_motion` had no caller in any windowed path, which left every CSS
  transition frozen in a real window while passing every headless test"*. The file
  is the memory of why the windowed loop looks the way it does.
- **`frame_pipeline.rs`, `arrange_phase.rs`, `style_phase.rs`, `compose_phase.rs`**
  — the repeated *"disjoint fields: a legal split borrow"* notes and the
  *"folding these into a struct would only move the same fifteen names"* refusals
  above each `#[allow(clippy::too_many_arguments)]`. Each restates its argument
  list, which is why they read as narration — but each is the standing answer to
  the exact change a reader is most likely to propose.
