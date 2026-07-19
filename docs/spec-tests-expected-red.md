# Spec-derived tests: the expected-red list

`cargo test --workspace --no-fail-fast` is **red on purpose**, and this file is what
makes that legible. Everything below is a spec requirement with no implementation
behind it yet. **Anything red that is not on this list is a regression.**

Use `--no-fail-fast`: without it cargo stops at the first failing target and the run
reports ~1,374 of 5,327 tests, which reads like catastrophe and is an artefact.

Current state at the pilot's landing: **5,304 passing, 128 failing.**

## Why these tests are different

They were derived from the specification by an agent that never opened `crates/`, then
translated by a second agent forbidden from changing any expected value. That
separation is the whole point. The existing 5,300-odd tests were written while looking
at the implementation, and the conformance campaign found them asserting a bug as
intended behaviour (`dsl_macro.rs:551`), passing with the entire frame pipeline
deleted, and passing with a whole font stack reverted. **A test written from the code
cannot catch the code being wrong — it defends it.** One such test had to be deleted
before `{user.name}` could be fixed, because fixing the bug turned a green test red.

Code-derived tests are conservative: they preserve. Spec-derived tests are corrective:
they judge. The first are worth having once the second have said what is correct.

## What the pilot found

55 cases for §4, of which 31 passed and 24 failed. Triage classified 18 failures as
real gaps, 4 as tests that misread the spec, and confirmed 3 **vacuous passes** in the
existing behaviour surface — tests that stay green when the feature is removed.

The signature result, and the argument for scaling this to the other 113 sections:
**four separate failures were one missing property.** The `flex` shorthand does not
exist, so `flex: 1 1 0px` is silently dropped. A code-derived test never writes it —
the existing suite writes the longhands, because that is what the code models — and an
audit of the implementation cannot see an absent property, because absent surface has
no code to audit. **This method audits the negative space.** Most of the rest is the
same shape: `clamp(none, …)`, physical units (`cm`/`mm`/`in`/`pt`/`pc`/`Q`), container
query units, `:is()`/`:not()`/`:where()`/`:has()`, `:nth-child(… of S)`,
`display: contents`, `transition-delay`, and four `animation-*` longhands. Missing
surface, not wrong logic.

## Triage is a mandatory gate, not a formality

Four failure messages read as algorithm bugs in code that is provably correct — the
flexbox §9.7 freeze loop, the `space-between` negative-free-space fallback, the grid
fr-restart search. **Acting on any of them would have damaged working code.** Two more
failed one layer below where the case aimed. Do not scale the derivation without
scaling the triage.

## And the isolation can leak

One case expected `10ch` to be 120px at `font-size: 20px` — which is 10 × 0.6em, and
0.6 is `MockTextShaper::ADVANCE_RATIO`. No reading of css-values-4 produces that
number; it came from the implementation. The requirement was in fact already met. That
case now asserts the *relationship* the spec states (a measurable `0` does not use the
0.5em substitute) instead of a constant, and it passes. Watch for expected values that
could only have come from the code.

## The list

- `a_flex_grow_factor_sum_below_one_leaves_space_unfilled`
- `a_max_width_violation_refreezes_and_redistributes`
- `a_negative_transition_delay_starts_part_way_through`
- `a_reversed_transition_is_shortened_by_its_progress`
- `a_transform_establishes_the_containing_block_for_fixed_descendants`
- `absolute_length_units_are_anchored_to_the_css_pixel`
- `align_content_stretch_grows_lines_by_equal_amounts`
- `alternate_iteration_parity_counts_from_one`
- `an_fr_track_blown_by_content_restarts_the_fr_algorithm`
- `an_infinitely_growable_track_absorbs_the_whole_max_content_remainder`
- `an_overflowing_item_ignores_its_auto_margins`
- `backwards_fill_uses_the_to_keyframe_when_the_direction_is_reversed`
- `clamp_lets_the_minimum_win_over_an_inverted_maximum`
- `color_interpolation_is_premultiplied`
- `container_query_units_fall_back_to_the_small_viewport`
- `display_contents_affects_the_box_tree_only`
- `forwards_fill_uses_the_end_of_the_last_completed_iteration`
- `is_and_not_take_the_specificity_of_their_most_specific_argument`
- `logical_block_start_maps_by_writing_mode`
- `nth_child_of_s_filters_before_counting`
- `overflow_clip_is_not_a_scroll_container_but_hidden_is`
- `space_between_falls_back_to_start_on_negative_free_space`
- `the_automatic_minimum_size_depends_on_overflow`
