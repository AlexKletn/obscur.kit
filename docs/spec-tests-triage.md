# Spec-derived tests: triage register

This file records **which red is which**, because they were not the same thing and
must not have been acted on the same way. It is now a closed register.

> **State (2026-07-31, measured at `cff9051`): 6,013 passing, 0 failing, 9 ignored.**
> The suite is no longer red on purpose. The last two — the owner-deferred env
> safe-area insets — were closed by `f11bb60`, which built the host→runtime inset
> supply the deferral said was missing; both tests pass unmodified
> (`crates/facade/obscur-kit/tests/spec_ch6b_insets.rs:53,82`). **Anything red is
> now a regression.**

History: **111 red at derivation**; 19 at the start of the 2026-07-20/21 campaign;
2 after it; 0 today. 310 cases were derived across §1–§7 and §9; §8 was skipped (67
sections of declared stubs, nothing to test). The former red list, with its counting
and baseline caveats, is at the end of this file.

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

## The red list (2)

**Anything red not on this list is a regression.** git-stash verified each batch.

State: **5,619 passing, 2 failing** (2026-07-21) — the campaign's landing. Every
buildable red had been closed; the 2 remaining were the owner-deferred env safe-area
tests, closed later by `f11bb60`. As of 2026-07-31 the list is empty: **6,013 passing,
0 failing.**

**Closed in the 19 → 2 campaign (2026-07-20/21), each by an implementation change with no
spec assertion weakened:**
- `border_inline_start_width_maps_to_the_right_edge_in_rtl` — logical border-* physical mapping (017449a).
- `a_newer_ir_version_is_refused_with_a_precise_diagnostic` — IR version range check (8387a55).
- `im_03_a_2x_asset_has_a_halved_logical_intrinsic_size` — replaced-element auto sizing reaches layout (8387a55).
- `ft_01_the_default_provider_renders_in_an_isolated_environment` — bundled NotoSans fallback; §5.18 deviation reversed (fe5f3f7).
- `d_an_absent_power_provider_reports_typed_absence_without_fabricating` — NullPowerProvider → PowerSource::Unknown (eb0054d).
- `an_exported_style_part_crosses_the_scope_boundary_and_an_unexported_one_does_not` — §4.16 ::part() matching (d94fd53).
- `ch4_16_a_generated_content_paints_without_becoming_a_node` — public tree children() is element-only, text is content (f15ddbb).
- `ch4_16_c_the_placeholder_is_an_exported_part_not_a_child` — placeholder is a synthetic part, not a child (de10c8e).
- `a_custom_loader_is_never_invoked_inside_the_frame_phase` — no-executor loads defer out of the frame phase; frame purity won the spec-vs-spec conflict, 9 peer tests amended to poll (2aa3bff).
- `logical_block_start_maps_by_writing_mode` — a modelled writing-mode + vertical block layout (a3873d7).
- `virtualfor_realizes_a_count_independent_of_the_item_count` — real VirtualLayout windowing, not a For in disguise (323edf1).
- `ch4_4_d_visibility_and_intersection_observers_track_the_scroll_chain` — public layout geometry is scroll-chain-relative (90c5c68; offset-0 half was the earlier bdeefad intersection fix).

**Also closed after the first triage (owner chose to build the deferred clusters):**
- `the_same_fault_carries_one_stable_code_across_two_contexts` — per-node style-value faults, one stable code, node context, property-independent message (289a5f2).
- `arch_17_a_missing_resource_diagnostic_carries_all_five_required_fields` — the resource diagnostic carries node + window context, stable code, recovery; poll on the frame-purity model (105b9d1).
- `arch_18_the_stable_code_survives_locale_and_call_site` — one stable code across locales/call sites + a scoped locale-keyed diagnostic message (en/ru/und), NOT the number/date i18n system (105b9d1).
- `an_ungranted_required_capability_refuses_instantiation` — ComponentRegistry granted-set; create_instance refuses a Required-but-ungranted capability (a34ab2b).
- `a_synthetic_event_is_untrusted_and_a_gated_default_action_does_not_run` — the click default action fires only for Trusted events; trust model = platform + first-party headless harness trusted, public programmatic API + external DevTools inject untrusted (a34ab2b).

**The 2 remaining — owner-deferred, and since closed:**
- ~~`env_safe_area_inset_top_resolves_to_the_host_inset`, `env_safe_area_inset_bottom_resolves_to_the_host_inset`~~ — the deferral read: no host-inset supply API; env() correctly resolves to 0; §6.15 says 44/34 are the host's report. Owner ruled keep red, defer. The entry named its own exit — "needs a host→runtime typed-env injection API to close without editing the test" — and `f11bb60` built exactly that. **Both green, neither test edited.**
