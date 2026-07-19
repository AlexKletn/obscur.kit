# Spec-derived tests: triage register

`cargo test --workspace --no-fail-fast` is red on purpose. This file records **which
red is which**, because they are not the same thing and must not be acted on the same
way.

State: **5,429 passing, 111 failing.** 310 cases were derived across §1–§7 and §9;
§8 was skipped (67 sections of declared stubs, nothing to test).

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

## The red list (75)

These are the implementation's to answer. **Anything red that is not on this list is a
regression.** Run with `--no-fail-fast` or cargo stops at the first red target.

State: **5,463 passing, 75 failing.**

Two of them were weakened by the correction pass and **restored by decision**, and the
reasoning matters more than the tests:

- `an_ungranted_required_capability_refuses_instantiation`
- `a_synthetic_event_is_untrusted_and_a_gated_default_action_does_not_run`

The literal reading that weakened them is defensible — §2.13/§2.14 say a contract
*exports* and a registry *stores* capability requirements, never that they are checked;
§2.3's whole text is "- trusted/untrusted origin.", which requires the origin to be
*reported*. But neither enforcement exists, so weakening would have turned a known
access-control gap from visibly red into silently green. That is the exact defect this
suite was built to remove. The general ruling that the spec outranks a test was given
on examples about version-query APIs and template tags; it was not extended to a trust
boundary or a capability check.

Their shape is itself the finding: `ComponentRegistry::create_instance` takes **no grant
set at all**, so there is no seam through which a capability could be checked. The
requirement is stored and never read.

- `a_click_handlers_commit_is_painted_by_the_frame_that_dispatched_it`
- `a_custom_loader_is_never_invoked_inside_the_frame_phase`
- `a_flex_grow_factor_sum_below_one_leaves_space_unfilled`
- `a_max_width_violation_refreezes_and_redistributes`
- `a_mutating_method_call_is_rejected_as_mutation`
- `a_negative_transition_delay_starts_part_way_through`
- `a_newer_ir_version_is_refused_with_a_precise_diagnostic`
- `a_prop_type_mismatch_is_reported_per_attribute_at_its_own_span`
- `a_reversed_transition_is_shortened_by_its_progress`
- `a_string_named_handler_is_refused_even_when_the_method_exists`
- `a_synthetic_event_is_untrusted_and_a_gated_default_action_does_not_run`
- `a_touch_leaves_no_persistent_hover_after_release`
- `a_transform_animation_moves_paint_without_re_running_layout`
- `a_transform_establishes_the_containing_block_for_fixed_descendants`
- `a_url_background_without_a_host_loader_fails_terminally_and_still_paints`
- `absolute_length_units_are_anchored_to_the_css_pixel`
- `align_content_stretch_grows_lines_by_equal_amounts`
- `alternate_iteration_parity_counts_from_one`
- `an_animated_opacity_never_reaches_computed_style`
- `an_exported_style_part_crosses_the_scope_boundary_and_an_unexported_one_does_not`
- `an_fr_track_blown_by_content_restarts_the_fr_algorithm`
- `an_infinitely_growable_track_absorbs_the_whole_max_content_remainder`
- `an_overflowing_item_ignores_its_auto_margins`
- `an_unexported_function_call_is_rejected_and_an_exported_one_is_not`
- `an_ungranted_required_capability_refuses_instantiation`
- `arch_16_a_malformed_declaration_kills_neither_its_rule_nor_the_sheet`
- `arch_17_a_missing_resource_diagnostic_carries_all_five_required_fields`
- `arch_18_the_stable_code_survives_locale_and_call_site`
- `arch_20_a_missing_image_falls_back_without_disturbing_the_column`
- `arch_28_a_missing_resource_does_not_trip_the_component_boundary`
- `backwards_fill_uses_the_to_keyframe_when_the_direction_is_reversed`
- `border_inline_start_width_maps_to_the_right_edge_in_rtl`
- `clamp_lets_the_minimum_win_over_an_inverted_maximum`
- `color_interpolation_is_premultiplied`
- `container_query_units_fall_back_to_the_small_viewport`
- `destroying_an_owner_destroys_the_windows_it_owns`
- `display_contents_affects_the_box_tree_only`
- `ed_01_backspace_deletes_one_grapheme_cluster`
- `ed_02_home_and_end_are_visual_line_not_paragraph`
- `fm_01_a_controlled_value_does_not_drift`
- `fm_03_readonly_is_not_disabled`
- `forwards_fill_uses_the_end_of_the_last_completed_iteration`
- `ft_01_the_default_provider_renders_in_an_isolated_environment`
- `headless_reports_always_on_top_unsupported_but_size_native_or_emulated`
- `im_02_object_fit_contain_geometry_and_position`
- `im_03_a_2x_asset_has_a_halved_logical_intrinsic_size`
- `ime_01_a_preedit_is_never_in_the_value`
- `input_is_serviced_before_resource_completion_in_one_frame`
- `inset_inline_start_maps_to_the_right_edge_in_rtl`
- `inspect_picking_honours_stacking_order`
- `io_and_direct_service_access_are_each_rejected_by_name`
- `io_async_and_direct_service_access_are_each_refused_by_category`
- `is_and_not_take_the_specificity_of_their_most_specific_argument`
- `logical_block_start_maps_by_writing_mode`
- `margin_inline_start_maps_to_the_right_edge_in_rtl`
- `motion_runs_without_any_stylesheet`
- `nth_child_of_s_filters_before_counting`
- `only_explicitly_exported_pure_functions_may_be_called`
- `overflow_clip_is_not_a_scroll_container_but_hidden_is`
- `padding_inline_start_maps_to_the_right_edge_in_rtl`
- `rs_02_no_loader_means_no_implicit_network_access`
- `rs_03_a_failed_load_is_a_state_not_a_panic_and_not_a_retry_loop`
- `sc_02_outer_shadow_is_clipped_out_of_the_border_box`
- `sc_04_gradient_stops_interpolate_premultiplied`
- `sc_05_conic_gradient_is_a_scene_brush_and_rasterizes`
- `sc_06_mix_blend_mode_multiply_exists`
- `space_between_falls_back_to_start_on_negative_free_space`
- `the_automatic_minimum_size_depends_on_overflow`
- `the_same_fault_carries_one_stable_code_across_two_contexts`
- `three_pending_reasons_produce_exactly_one_frame`
- `three_requests_and_two_commits_coalesce_into_one_frame`
- `tx_02_a_canonically_equivalent_cluster_becomes_one_glyph`
- `tx_03_collapsed_spaces_have_zero_advance_and_keep_their_offsets`
- `tx_05_bidi_selection_is_disjoint_and_runs_reorder`
- `virtualfor_realizes_a_count_independent_of_the_item_count`
