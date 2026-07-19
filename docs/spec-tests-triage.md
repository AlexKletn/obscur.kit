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

## Confirmed real gaps (48)

These are the implementation's to answer. Do not weaken them.

- **P6-01 / spec_ch6_windows.rs::headless_reports_always_on_top_unsupporte** _(§6 Platform, windows & s)_ — NullWindowBackend never discriminates the §6.2 capability categories on the property-update path: update_window returns Performed(Native) for every delta and applies every property, includin
- **P6-42 / spec_ch6_windows.rs::destroying_an_owner_destroys_the_windows_** _(§6 Platform, windows & s)_ — NullWindowBackend accepts a WindowRelation::owned_by / Modality relation at creation and honors it in no code path whatsoever — not on destroy, not on input, not on z-order. Closing the owne
- **P6-06b — RECAST, not the failing assertion (WinitWindowBackend::set_pr** _(§6 Platform, windows & s)_ — The one backend that actually implements presentation silently drops PresentationMode::restore_previous_state and still reports Performed(Native). Leaving presentation resets state to Normal
- **S3-02 three_requests_and_two_commits_coalesce_into_one_frame** _(§3 State, transactions &)_ — One frame per distinct pending FrameReason. The scheduler coalesces same-reason requests only, so N distinct reasons pending between two frame boundaries cost N frames. 3x ExternalRender + 2
- **S3-27 input_is_serviced_before_resource_completion_in_one_frame** _(§3 State, transactions &)_ — Same defect as S3-02, second symptom: Input + ResourceCompletion pending together produce 2 frames. The frame that served Input left a second frame pending for the resource reason.
- **S3-35 three_pending_reasons_produce_exactly_one_frame** _(§3 State, transactions &)_ — Same defect as S3-02, third symptom: Input + UiCommit + Animation pending together produce 3 frames.
- **S3-05 a_click_handlers_commit_is_painted_by_the_frame_that_dispatched_** _(§3 State, transactions &)_ — Universal one-frame input lag on the reactive path: the view builder is called BEFORE the frame's input-dispatch phase, so a handler's state mutation cannot reach the tree until frame N+1. T
- **S3-07 a_transform_animation_moves_paint_without_re_running_layout** _(§3 State, transactions &)_ — `transition: transform 1s linear` never plays. `transform` is not in the animatable-property set, so no CSS transition timeline is ever registered and the property snaps to its end value on 
- **S3-36 a_custom_loader_is_never_invoked_inside_the_frame_phase** _(§3 State, transactions &)_ — With no blocking executor installed (the default for a bare UiRuntime), a custom ResourceLoader is called synchronously on the frame thread, inside the frame phase.
- **arch-16** _(§1 Architecture & bounda)_ — REAL GAP (emission). The style subsystem recovers from an invalid declaration correctly but emits no Diagnostic. §1.6 rules: "failures should be localized to the smallest safe scope" / "diag
- **arch-17** _(§1 Architecture & bounda)_ — REAL GAP on the count assertion (0 emitted, 1 required) — same single defect as arch-16, in the resources subsystem. BUT the case carries a BAD SUB-ASSERTION that must be corrected before it
- **arch-18** _(§1 Architecture & bounda)_ — REAL GAP on the three count assertions (0 emitted, 1 required each) — same defect. BUT the case's FINAL assertion is invented and must be deleted: `assert_ne!(a[0].message, b[0].message, "th
- **arch-20** _(§1 Architecture & bounda)_ — REAL GAP, cleanest instance of the defect. §1.6 "fallback and recovery are preferred where correctness can be maintained" is satisfied behaviourally — the frame is balanced, all three cells 
- **arch-28** _(§1 Architecture & bounda)_ — REAL GAP, and the most valuable of the six because it is discriminating rather than merely additive. §1.7 "Layout, renderer and resource failures use subsystem-specific recovery boundaries" 
- **NG-MOTION-01 — an_animated_opacity_never_reaches_computed_style** _(§9 Roadmap & non)_ — The motion engine writes its sampled presentation value into the node's ComputedStyle, collapsing §4.17 pipeline stages 1 and 2. A style read during an animation returns a value the author n
- **NG-MOTION-02 — motion_runs_without_any_stylesheet** _(§9 Roadmap & non)_ — Transform motion does not exist by ANY path — native or CSS. The runtime's animation binding can only carry a scalar, so a MotionValue::Transform track is silently dropped and never reaches 
- **NG-OKUI-CALL-01 — only_explicitly_exported_pure_functions_may_be_calle** _(§9 Roadmap & non)_ — §2.10's Allowed side is unimplemented: no declaration surface — existing or hypothetical — can make a function call legal in an .okui expression. Every call is refused unconditionally.
- **NG-OKUI-IO-01(real half) — direct service access is ACCEPTED, not refu** _(§9 Roadmap & non)_ — Discovered during triage of NG-OKUI-IO-01. `{ services.storage.get("k") }` compiles clean — has_errors=false, zero diagnostics, UI IR emitted — whenever `services` is a declared prop or stat
- **RTL-01 — spec_ch7_rtl::margin_inline_start_maps_to_the_right_edge_in_r** _(§7)_ — Block-level over-constrained margin resolution ignores `direction`. The logical→physical mapping itself is CORRECT — the test's own diagnosis ("the logical→physical mapping is unconditional"
- **RTL-01 — spec_ch7_rtl::border_inline_start_width_maps_to_the_right_edg** _(§7)_ — `border-inline-start` / `border-inline-start-width` / `border-inline-*` are entirely unimplemented — no PropertyId, no compiler entry, no effect in EITHER direction. Note the test's expected
- **DEV-01 — spec_ch7_devtools::inspect_picking_honours_stacking_order** _(§7)_ — Hit testing has no concept of z-order or paint order. The `z_order` fed to the spatial index is TREE DEPTH, and equal-depth ties are broken by ASCENDING NodeId — i.e. the FIRST sibling wins,
- **DIAG-01 — spec_ch7_diagnostics::the_same_fault_carries_one_stable_code** _(§7)_ — An unparseable declaration VALUE is discarded silently at every layer. It is not merely unrouted to the runtime's DiagnosticSink — the CSS front end never records it at all, so no typed reco
- **spec_ch2_expressions.rs :: a_mutating_method_call_is_rejected_as_mutat** _(§2 Components, tree & DS)_ — §2.10 Forbidden: "- arbitrary mutation;" and "- direct service access;" are not enforced at all. Every Rust method call is admitted wholesale, so any effectful call spelled as a method passe
- **spec_ch2_expressions.rs :: an_unexported_function_call_is_rejected_and** _(§2 Components, tree & DS)_ — The ALLOWED half of §2.10 is unimplemented. §2.10 Allowed ends with "- explicitly exported pure functions." There is no mechanism to declare a function template-exported, so a correctly-decl
- **spec_ch2_contracts.rs :: a_string_named_handler_is_refused_even_when_t** _(§2 Components, tree & DS)_ — §2.3: "Events are typed. String-based method dispatch is not used." and §2.13: "Unknown attributes, events and string method names are compile errors." Both sentences verified verbatim in th
- **spec_ch2_contracts.rs :: a_prop_type_mismatch_is_reported_per_attribut** _(§2 Components, tree & DS)_ — §2.9: "- typed props, events and commands;" and "- compile-time contract validation;" (verified, lines 149-150), with §2.8 "- typed props and events;" / "- compile-time validation;". Prop ty
- **spec_ch2_tree.rs :: virtualfor_realizes_a_count_independent_of_the_ite** _(§2 Components, tree & DS)_ — §2.11 Collections lists "- `VirtualFor`;" (verified, line 191). The front end refuses it outright. The deriver's cross-reference to §4.9 is also correct — I opened it: specs/.../04-layout-cs
- **spec_ch2_interaction.rs :: a_touch_leaves_no_persistent_hover_after_re** _(§2 Components, tree & DS)_ — §2.4 line 73, verbatim: "Touch does not synthesize persistent hover. Pen hover depends on device capability." The runtime computes the hover chain identically for every pointer device and ne
- **spec_ch2_lifecycle_ir.rs :: a_newer_ir_version_is_refused_with_a_preci** _(§2 Components, tree & DS)_ — §2.15, verified verbatim: "Compatibility includes: - supported version ranges; - migrations where practical; - capability negotiation; - precise diagnostics for newer unsupported IR." The de
- **spec_ch2_contracts.rs :: an_exported_style_part_crosses_the_scope_boun** _(§2 Components, tree & DS)_ — §2.12: "Components may declare: … - exported style parts;" (verified, line 206). `StylePartContract` is stored metadata that no styling path consumes: there is no `::part()` in the selector 
- **NOT IN THE FAILURE LIST — found via the vacuity sweep: §2.5 focus-visi** _(§2 Components, tree & DS)_ — §2.5: "Focus is a first-class subsystem: … - focus-visible semantics;" and "Focus is not modeled as a CSS flag. CSS consumes focus state produced by the focus subsystem." Nothing in the code
- **SC-02** _(§5 Rendering, text & res)_ — An outer box-shadow is not knocked out of the border box: it fills the whole spread-inflated rounded rect, so a translucent background composites over the shadow and is tinted by it. css-bac
- **SC-04** _(§5 Rendering, text & res)_ — REAL, but not for the reason the case names. `linear-gradient(to bottom, transparent, rgb(0,0,255))` reaches the display list correctly as a Brush::Linear; the CPU backend then degrades EVER
- **SC-05** _(§5 Rendering, text & res)_ — `conic-gradient` never survives CSS compilation because ANGULAR colour stops are rejected. The engine models conic gradients end to end (ConicGradient value type, absolutize, Brush::Conic, c
- **SC-06** _(§5 Rendering, text & res)_ — `mix-blend-mode` is missing surface, not wrong logic. `BlendMode` exists as a scene type and `SceneLayer` carries one, but the string `mix-blend-mode` appears nowhere in the CSS property tab
- **TX-02** _(§5 Rendering, text & res)_ — There is no Unicode normalization / canonical-equivalence step anywhere in the text stack, so css-fonts-4 §5.3's second matching bullet is unimplemented: "If a sequence of multiple codepoint
- **TX-03** _(§5 Rendering, text & res)_ — REAL in both halves, though the case drove a layer that does no collapsing at all. (a) `ParagraphShaper::shape_paragraph` + `ParagraphLayout::wrap` — the surface the case calls — never colla
- **TX-05** _(§5 Rendering, text & res)_ — Selection geometry has zero bidi awareness. `SelectionGeometry::from_paragraph` emits one rect PER LINE, from caret-x(lo) to caret-x(hi), and computes each pen x by summing the advances of e
- **ED-01** _(§5 Rendering, text & res)_ — `DeletionUnit::Grapheme` is implemented as previous/next CHAR boundary, not extended grapheme cluster. The enum's own doc says "One extended grapheme cluster", so this is a documented-intent
- **ED-02** _(§5 Rendering, text & res)_ — Home/End are `\n`-scoped, i.e. paragraph/logical-line, with no access to line boxes at all. On soft-wrapped text Home from the start of visual line 1 jumps to 0. FLAG: §5.11 says only "- wor
- **IME-01** _(§5 Rendering, text & res)_ — A live preedit is never painted when the field's value is EMPTY. The Input widget's only text node lives inside a `.placeholder` overlay; `EditingRole::Placeholder.overrides()` gates on `!vi
- **FM-01** _(§5 Rendering, text & res)_ — A truly controlled field is impossible: the reseed discriminator short-circuits when the retained value has not CHANGED. §3.1 says "values flow down" with no two-way binding, and §5.14 lists
- **FM-03** _(§5 Rendering, text & res)_ — Clicking a disabled field CLEARS focus instead of leaving it. The implementation's own comment says the opposite of what the code does — strong independent evidence this is a defect rather t
- **IM-02** _(§5 Rendering, text & res)_ — `object-position` is missing surface: the identifier does not exist as a PropertyId, in the CSS compiler, or in the coordinator. The declaration is dropped at parse and the tile is unconditi
- **IM-03** _(§5 Rendering, text & res)_ — REAL, but only one of its two halves. The UNAMBIGUOUS defect is that `width:auto;height:auto` on an Image resolves to (400, 0) — the auto HEIGHT collapses to zero, so replaced-element intrin
- **RS-02** _(§5 Rendering, text & res)_ — A resource that fails to load produces no diagnostic of any kind — the failure is recorded as hub state and nothing else. §1.6 names "invalid content or resource" as a failure class and requ
- **RS-03** _(§5 Rendering, text & res)_ — Same missing emission as RS-02, on the decode-failure path. The no-panic and no-retry halves both pass (loader invoked once over five frames), so the only unmet requirement is the §1.6 diagn
- **FT-01** _(§5 Rendering, text & res)_ — CONFIRMED, and confirmed as a KNOWN gap rather than a discovery, exactly as the translator flagged. §5.18 requires a "bundled fallback set for WASM/isolated environments" and "usable renderi

## Disputed — do NOT act on these (45)

Triage judged each of these to misread the spec, to test through a broken fixture, or
to assert against the wrong configuration. **Acting on any of them would damage working
code.** Each needs a decision — correct the case, or overrule the triage — and per the
working rule on this suite, that decision is the owner's, not an agent's.

- **P6-03 / spec_ch6_windows.rs::size_requests_are_clamped_by_min_max_cons** _(§6 Platform, windows & s)_ — BOTH halves are invented, and the failure message is actively misleading. (1) The message says 'constraints stored but never applied'. That is false: WindowConstraints::constrain is correct 
- **P6-06 / spec_ch6_windows.rs::exiting_fullscreen_restores_position_size** _(§6 Platform, windows & s)_ — The failing assertion — '§6.5: borderless fullscreen is the emulatable form' — is not in the spec. §6.5 reads in full: 'Modes may include:' / '- normal;' / '- borderless;' / '- exclusive whe
- **P6-19 / spec_ch6_routing.rs::percent_encoding_is_decoded_after_segment** _(§6 Platform, windows & s)_ — §6.13 never mentions percent-encoding, decoding, or encoding at all. Its complete text is 'Basic URL-like router:' followed by '- path;' '- query;' '- fragment;' '- route patterns;' '- typed
- **P6-15 / spec_ch6_services.rs::opening_an_unhandleable_uri_is_never_rep** _(§6 Platform, windows & s)_ — The subject is a test double and the case tests the mock — the translator's own category (C), applied here without being noticed. RecordingShellProvider is documented at shell_provider.rs:57
- **P6-30 / spec_ch6_services.rs::headless_window_capture_is_unsupported_b** _(§6 Platform, windows & s)_ — 'Node capture needs no compositor' appears nowhere in §6.17. The section reads: 'Capabilities:' / '- capture window;' / '- capture surface;' / '- capture node;' / '- capture image;' / '- mar
- **S3-09 a_batch_with_one_invalid_op_applies_none_of_it** _(§3 State, transactions &)_ — §3.2 says exactly (03-state-transactions-and-runtime.md:24, verbatim): "Mutations are normalized into validated `UiOps` and committed atomically between frames." and (lines 37-41) "Rollback 
- **arch-02** _(§1 Architecture & bounda)_ — BAD TEST. §1.2 verbatim: "Independent version domains:" then the seven bullets. That is a *policy* statement — these things version independently of one another and of the kit. It says nothi
- **arch-03** _(§1 Architecture & bounda)_ — BAD TEST on its first assertion, which is the one that fired. §1.3's only normative sentence is "Unused backends and optional integrations must not be linked." The section lists *crates* (in
- **arch-29** _(§1 Architecture & bounda)_ — BAD TEST, and the dangerous one — this is the type-(b) failure whose message reads as a layout-subsystem bug. The translator's own note 1 predicted the hazard; I confirmed it empirically and
- **NG-URLIMG-01 — a_url_background_without_a_host_loader_fails_terminally** _(§9 Roadmap & non)_ — §5.15 (05-rendering-text-and-resources.md:301-333) lists "- loading/ready/failed states;" (:317) among the hub's *Features* and "- URL through user/host loader;" (:330) among its *Sources*, 
- **NG-OKUI-ASSIGN-01 — assignment_inside_an_interpolation_is_a_compile_er** _(§9 Roadmap & non)_ — §2.10 (02-components-tree-and-dsl.md:171) says only "- assignment;" is Forbidden; §2.13 (:222) says "Unknown attributes, events and string method names are compile errors." Neither says an o
- **NG-OKUI-UNKNOWN-01 — unknown_events_attributes_and_elements_are_compil** _(§9 Roadmap & non)_ — §2.13 (02-components-tree-and-dsl.md:222) is one sentence: "Unknown attributes, events and string method names are compile errors." That is the whole requirement, and the engine meets it — t
- **NG-OKUI-IO-01 — io_async_and_direct_service_access_are_each_refused_by** _(§9 Roadmap & non)_ — §2.10 (02-components-tree-and-dsl.md:169-176) is a list of forbidden *things*, not a list of required *messages*. It nowhere states that a refusal must be attributed to its category, and §1.
- **RTL-01 — spec_ch7_rtl::padding_inline_start_maps_to_the_right_edge_in_** _(§7)_ — The expected 180 is arithmetically impossible for a padding declared on the CONTAINER, and the logical mapping it accuses is provably correct. With `box-sizing: content-box; width: 300px`, `
- **RTL-01 — spec_ch7_rtl::inset_inline_start_maps_to_the_right_edge_in_rt** _(§7)_ — The observed -20 is the CORRECT answer, and the test's diagnosis ("mirrored the wrong way and off the container") is wrong. specs/w3c/css-position-3.txt:792-794, quoted myself: "If only one 
- **HL-01 — spec_ch7_headless::the_clock_is_sampled_not_accumulated** _(§7)_ — The cited clause forbids frame-rate-dependent SLOWING, not f32 rounding. §3.5, specs/obscur-kit-2-spec-drafts/03-state-transactions-and-runtime.md:107, quoted myself: "animation progress is 
- **HL-05 — spec_ch7_headless::a_dispatch_result_reports_a_focus_change_on** _(§7)_ — The test reads `focus_changed` off the wrong dispatch. Its `click` closure (spec_ch7_headless.rs:217-226) issues `pointer_down` then `pointer_up` and returns the outcome of the UP. Focus mov
- **L10N-09 — spec_ch7_localization::fallback_is_resolved_per_key_not_per_** _(§7)_ — The "default locale en" is invented, and the test never configures one. §7.2, specs/obscur-kit-2-spec-drafts/07-...md:31-39 lists "- fallback;" as a bare bullet — no default-locale layer, no
- **TEL-03 — spec_ch7_diagnostics::deep_tracing_when_enabled_covers_dispat** _(§7)_ — The test never dispatches anything, so no dispatch span could exist under any implementation. Its body (spec_ch7_diagnostics.rs:206-208) is ten bare `rt.render_now(...)` calls — no `handle_p
- **SEC-04 — spec_ch7_endpoint_security::limits_are_enforced_without_colla** _(§7)_ — The denial is documented, deliberate, and correct for the environment the test built. crates/diagnostics/obscur-devtools-server/src/endpoint.rs:252-262 denies a Subscribe when `!self.event_p
- **SNAP-02 — spec_ch7_snapshot_replay::a_snapshot_records_focus_selection** _(§7)_ — `.panel` is not a scroll container, so reading back 0.0 is correct. The test's stylesheet (spec_ch7_snapshot_replay.rs:20-22) is `.panel { width: 400px; height: 300px }` — no `overflow` decl
- **GOLD-01 — spec_ch7_goldens::a_raster_tolerance_must_be_explicit_and_a_** _(§7)_ — Pixel-coordinate reporting is invented. §7.14 is two sentences, specs/obscur-kit-2-spec-drafts/07-...md:281, quoted in full: "Raster golden tests are allowed with explicit tolerances for ant
- **spec_ch2_tree.rs :: tree_key_reorder_preserves_child_state** _(§2 Components, tree & DS)_ — §2.2 "child identity is stable through explicit keys" IS honoured. The test's own harness breaks its premise: `rows()` builds each row as `keyed(100 + i as u64, "row", key, …)`, so the Compo
- **spec_ch2_tree.rs :: tree_key_removal_does_not_reuse_the_removed_id** _(§2 Components, tree & DS)_ — Same harness defect, same file: `100 + i` shifts every row's ComponentId when an earlier sibling is dropped, so "c" is component 102 in frame 1 and component 101 in frame 2 and cannot reuse 
- **spec_ch2_lifecycle_ir.rs :: an_error_boundary_isolates_the_failing_sib** _(§2 Components, tree & DS)_ — The boundary works; the case sampled one frame too early. §1.7 (which the case itself cites) lists four consequences and the fourth is "- may be retried according to component or host policy
- **spec_ch2_interaction.rs :: the_selection_anchor_persists_across_succes** _(§2 Components, tree & DS)_ — The case uses `SelectionPolicy::default()`, which is SINGLE selection (selection_policy.rs:124-128: "Defaults to Single with allow_empty on"). For Single, `apply_range` correctly collapses a
- **spec_ch2_contracts.rs :: a_default_slot_and_a_named_slot_partition_the** _(§2 Components, tree & DS)_ — I read §2.12 myself, in full: "Templates are stateless and lifecycle-free. / Components may declare: - named slots; - allowed child contracts; - slot cardinality; - exported style parts; - e
- **spec_ch2_contracts.rs :: a_contract_declared_default_is_applied_rather** _(§2 Components, tree & DS)_ — §2.13 says only "- defaults;". The case is unsatisfiable as written, by ANY implementation: `#[contract(default)]` takes no value, and the values it expects ("primary", 5) appear nowhere exc
- **spec_ch2_contracts.rs :: a_template_may_not_hold_state_or_a_lifecycle_** _(§2 Components, tree & DS)_ — §2.12's "Templates are stateless and lifecycle-free." is a property of the template concept, not a requirement that `.okui` expose a `<template name=…>` tag with `<state>` and `<on:mount>` c
- **spec_ch2_interaction.rs :: a_synthetic_event_is_untrusted_and_a_gated_** _(§2 Components, tree & DS)_ — §2.3's entire text on this is one bullet in the Event API list: "- trusted/untrusted origin." It requires the origin to be REPORTED and says nothing about who gates on it. The case's own tru
- **spec_ch2_lifecycle_ir.rs :: an_ungranted_required_capability_refuses_i** _(§2 Components, tree & DS)_ — §2.14, verbatim: "`ComponentRegistry` stores: - stable component ID; - contract schema; - version; - capability requirements; - factory; - diagnostics metadata." STORES. §2.13 likewise only 
- **spec_ch2_lifecycle_ir.rs :: an_ir_document_with_no_version_field_is_re** _(§2 Components, tree & DS)_ — §2.15 says "- versioned;" and "- validated;"; it does not describe a document that omits its version. The scenario is not constructible in this codec: the header is magic(4) + format u16(2) 
- **spec_ch2_expressions.rs :: assignment_in_an_expression_is_rejected** _(§2 Components, tree & DS)_ — §2.10 enumerates forbidden FORMS; it says nothing about diagnostic wording. `{count = count + 1}` IS refused (verified through the public front end) — with the generic message, which even co
- **spec_ch2_expressions.rs :: awaiting_in_an_expression_is_rejected_as_as** _(§2 Components, tree & DS)_ — §2.10 Forbidden "- async work;" is enforced: `{user.load().await}` produces a diagnostic (verified). The case fails only because that diagnostic does not contain the word "async". No clause 
- **spec_ch2_expressions.rs :: io_and_direct_service_access_are_each_rejec** _(§2 Components, tree & DS)_ — SPLIT VERDICT — do not delete this case, split it. The I/O half is a wording-only failure: `{read_file("config.toml")}` IS refused (as a free call) and merely fails a `.contains("I/O")` grep
- **spec_ch2_expressions.rs :: a_block_of_statements_is_not_permitted_in_o** _(§2 Components, tree & DS)_ — §2.9 "- no embedded general-purpose scripting language;" is honoured — the block is refused and produces no artifact. The case fails on wording: the refusal arrives as "this is not a valid e
- **spec_ch2_expressions.rs :: an_unexported_function_call_is_rejected_and** _(§2 Components, tree & DS)_ — The `fmt_price` half of this case is a REAL GAP (listed above). The `fmt_tax` half is wording-only: the call is refused, the diagnostic just does not echo the callee's name. §2.10 requires t
- **spec_ch2_contracts.rs :: a_contract_diagnostic_carries_the_author_s_li** _(§2 Components, tree & DS)_ — The two clauses cited are §2.8 "- stable source mapping;" (a goal of the Rust DSL) and §2.15 "- source-mapped;" (a property of UI IR). Neither says line-and-column. The emitted diagnostic — 
- **spec_ch2_expressions.rs :: duplicate_keys_are_diagnosed_rather_than_co** _(§2 Components, tree & DS)_ — §2.11's text is "- mandatory stable `key`;" — mandatory (which IS enforced, see the passing `a_for_without_a_key_is_a_compile_error`) and stable. Nothing requires a duplicate-key diagnostic.
- **spec_ch2_tree.rs :: a_false_conditional_attribute_is_absent_not_empty** _(§2 Components, tree & DS)_ — §2.11's entire text here is one bullet: "- simple conditional attributes." It does not state HTML's boolean-attribute omission semantics, and `.okui`/`okui!` have no distinct conditional-att
- **SC-01** _(§5 Rendering, text & res)_ — The translator's suspicion is correct — this is a false positive about a legitimate lowering choice. I re-read the clause at specs/w3c/css-backgrounds-3.txt:2667-2670: "In terms of stacking 
- **SC-03** _(§5 Rendering, text & res)_ — The deriver over-read §5.7. The bullet is "- linear compositing for blending, blur and filters;" — it scopes linear light to BLENDING, BLUR and FILTERS. `opacity: 0.5` is none of those: it i
- **SC-10** _(§5 Rendering, text & res)_ — Wrong observation point, and the requirement is already met. §5.2's "Unsupported effects and fallbacks are explicit diagnostics" IS implemented: the coordinator emits a real BackdropFilter c
- **IME-05** _(§5 Rendering, text & res)_ — Unfaithful reduction — the translator flagged the substitution and it does not survive scrutiny. §5.12's "duplicate keydown/text delivery is avoided" concerns ONE physical press arriving on 
- **IM-03 (DPI half only)** _(§5 Rendering, text & res)_ — Listed under real_gaps for its auto-height defect; its DPI premise is separately wrong. §5.16 says only "- DPI variants;" — it does not say the `@Nx` suffix is parsed off an explicitly autho
