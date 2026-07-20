# Spec-derived tests: triage register

`cargo test --workspace --no-fail-fast` is red on purpose. This file records **which
red is which**, because they are not the same thing and must not be acted on the same
way.

State: **5,585 passing, 23 failing** (was 111 at derivation; the implementation phase is
closing them). 310 cases were derived across §1–§7 and §9; §8 was skipped (67 sections of
declared stubs, nothing to test). The current red list, with counting and baseline
caveats, is at the end of this file.

## Read this before closing anything

Triage classified the failures into two piles of nearly equal size — **48 real gaps
and 45 bad tests.** That ratio is the headline finding of the rollout, and it is very
different from the §4 pilot, which was 18 real gaps against 4 bad tests.

**The method's precision tracks the spec's precision.** §4 defers to the vendored W3C
texts: normative, numeric, quotable, so a deriver who never saw the code still lands
on the right expected value. §1, §2, §3 and §7 are this project's own prose, and much
of it is a bullet list — "assignment;" under Forbidden, "components produce a desired
declarative tree". A deriver reading that has to invent the observable specifics, and
inventing is exactly what produces a confident wrong test.

So: **the triage step is not optional, and the derivation is not trustworthy without
it.** Four §4 failures read as algorithm bugs in provably correct code; at this scale
there are 45.

## The 45 are not all the same defect

Sorting them by what actually went wrong matters more than the count:

1. **Invented requirements.** The spec says less than the test assumes. Example: §2.10
   lists "assignment" as forbidden; a case additionally demanded that the compiler
   suppress its output artifact, which no sentence requires. Correcting these means
   weakening the assertion to what the spec actually says.
2. **Broken fixtures.** The test's own harness contradicts its premise. Example:
   `tree_key_reorder_preserves_child_state` builds each row with `100 + i` as its
   ComponentId, so identity is a function of *position* — the very thing the test
   claims to hold constant. The implementation is correct; the fixture defeats itself.
3. **Wrong setup.** `the_selection_anchor_persists_across_successive_shift_clicks` uses
   the default selection policy, which is *Single*; collapsing a range is the
   documented, correct behaviour for it. The case wanted Multi and never asked for it.
4. **Sampled too early.** An error-boundary case read one frame before the retry
   §1.7 allows had landed.

Only (1) is a disagreement about what the spec means. (2)–(4) are ordinary test bugs —
and they are a consequence of the isolation, not a failure of it: the deriver never saw
the API, and the translator was forbidden from changing an expected value, so a wrong
fixture got translated faithfully.

## The red list (23)

**Anything red not on this list is a regression.** git-stash verified each batch.

State: **5,585 passing, 23 failing.**

Held to the owner, left red on purpose: arch_17 (context.window on an unbound runtime),
arch_18's message-localization assertion (§1.6 wants a stable code, not a translated
message). env(safe-area-inset-*) x2 are not-reachable: CSS env() is unimplemented.

- `a_custom_loader_is_never_invoked_inside_the_frame_phase`
- `a_newer_ir_version_is_refused_with_a_precise_diagnostic`
- `a_synthetic_event_is_untrusted_and_a_gated_default_action_does_not_run`
- `a_url_background_without_a_host_loader_fails_terminally_and_still_paints`
- `an_exported_style_part_crosses_the_scope_boundary_and_an_unexported_one_does_not`
- `an_ungranted_required_capability_refuses_instantiation`
- `arch_17_a_missing_resource_diagnostic_carries_all_five_required_fields`
- `arch_18_the_stable_code_survives_locale_and_call_site`
- `border_inline_start_width_maps_to_the_right_edge_in_rtl`
- `ch4_16_a_generated_content_paints_without_becoming_a_node`
- `ch4_16_b_scrollbar_parts_are_hidden_from_the_component_tree_but_in_devtools`
- `ch4_16_c_the_placeholder_is_an_exported_part_not_a_child`
- `ch4_4_d_visibility_and_intersection_observers_track_the_scroll_chain`
- `ch4_9_b_scroll_anchoring_compensates_growth_above_the_viewport`
- `d_an_absent_power_provider_reports_typed_absence_without_fabricating`
- `env_safe_area_inset_bottom_resolves_to_the_host_inset`
- `env_safe_area_inset_top_resolves_to_the_host_inset`
- `ft_01_the_default_provider_renders_in_an_isolated_environment`
- `im_03_a_2x_asset_has_a_halved_logical_intrinsic_size`
- `logical_block_start_maps_by_writing_mode`
- `sc_04_gradient_stops_interpolate_premultiplied`
- `the_same_fault_carries_one_stable_code_across_two_contexts`
- `virtualfor_realizes_a_count_independent_of_the_item_count`
